use std::path::{Path, PathBuf};
use std::time::Duration;

use tauri::{AppHandle, Runtime, Url, WebviewUrl, WebviewWindowBuilder};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub page_size: String,
    pub landscape: bool,
    pub margin_mm: f64,
}

pub async fn render_html_to_pdf<R: Runtime>(
    app: &AppHandle<R>,
    html_path: &Path,
    out_path: &Path,
    options: &ExportOptions,
) -> Result<(), String> {
    let canonical = html_path
        .canonicalize()
        .map_err(|e| format!("无法解析 HTML 路径: {e}"))?;
    let file_url = Url::from_file_path(&canonical)
        .map_err(|_| "无法构建 file:// URL（路径可能无效）".to_string())?;

    let label = format!("html-export-{}", Uuid::new_v4());
    let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::External(file_url))
        .visible(false)
        .inner_size(1200.0, 1600.0)
        .build()
        .map_err(|e| format!("创建导出 WebView 失败: {e}"))?;

    let export_result = async {
        window
            .navigate(
                Url::from_file_path(&canonical).map_err(|_| "无法构建 file:// URL".to_string())?,
            )
            .map_err(|e| format!("加载 HTML 失败: {e}"))?;

        tokio::time::sleep(Duration::from_millis(800)).await;
        inject_print_styles(&window, options)?;
        tokio::time::sleep(Duration::from_millis(200)).await;

        platform_print_pdf(&window, out_path).await
    }
    .await;

    let _ = window.close();
    export_result
}

fn inject_print_styles<R: Runtime>(
    window: &tauri::WebviewWindow<R>,
    options: &ExportOptions,
) -> Result<(), String> {
    let size = match options.page_size.to_ascii_uppercase().as_str() {
        "LETTER" => {
            if options.landscape {
                "letter landscape"
            } else {
                "letter"
            }
        }
        _ => {
            if options.landscape {
                "A4 landscape"
            } else {
                "A4"
            }
        }
    };
    let margin = options.margin_mm;
    let script = format!(
        r#"(function() {{
  const id = 'doc-agent-print-style';
  let el = document.getElementById(id);
  if (!el) {{
    el = document.createElement('style');
    el.id = id;
    document.head.appendChild(el);
  }}
  el.textContent = `@page {{ size: {size}; margin: {margin}mm; }}
@media print {{
  body {{ -webkit-print-color-adjust: exact; print-color-adjust: exact; }}
}}`;
}})();"#
    );
    window
        .eval(&script)
        .map_err(|e| format!("注入打印样式失败: {e}"))
}

async fn platform_print_pdf<R: Runtime>(
    window: &tauri::WebviewWindow<R>,
    out_path: &Path,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        return macos::print_pdf(window, out_path).await;
    }
    #[cfg(windows)]
    {
        return win::print_pdf(window, out_path).await;
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = (window, out_path);
        Err("html_to_pdf 仅支持 macOS 与 Windows".to_string())
    }
}

pub fn pdf_page_count(path: &Path) -> Result<u32, String> {
    let doc = lopdf::Document::load(path).map_err(|e| format!("读取 PDF 失败: {e}"))?;
    Ok(doc.get_pages().len() as u32)
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use block2::RcBlock;
    use objc2::MainThreadMarker;
    use objc2_foundation::NSData;
    use objc2_foundation::NSError;
    use objc2_web_kit::{WKPDFConfiguration, WKWebView};
    use std::sync::{Arc, Mutex};

    pub async fn print_pdf<R: Runtime>(
        window: &tauri::WebviewWindow<R>,
        out_path: &Path,
    ) -> Result<(), String> {
        let out = out_path.to_path_buf();
        let (tx, rx) = tokio::sync::oneshot::channel::<Result<Vec<u8>, String>>();
        let tx = Arc::new(Mutex::new(Some(tx)));

        window
            .with_webview({
                let tx = Arc::clone(&tx);
                move |wv| unsafe {
                    let wk: &WKWebView = &*(wv.inner().cast::<WKWebView>());
                    let mtm = MainThreadMarker::new().expect("with_webview runs on main thread");
                    let config = WKPDFConfiguration::new(mtm);

                    let block = RcBlock::new(move |data: *mut NSData, error: *mut NSError| {
                        let result = if !error.is_null() {
                            let desc = (*error).localizedDescription();
                            Err(format!("PDF 生成失败: {desc}"))
                        } else if data.is_null() {
                            Err("PDF 生成未返回数据".to_string())
                        } else {
                            Ok((*data).to_vec())
                        };
                        if let Some(sender) = tx.lock().ok().and_then(|mut g| g.take()) {
                            let _ = sender.send(result);
                        }
                    });

                    wk.createPDFWithConfiguration_completionHandler(Some(&config), &block);
                }
            })
            .map_err(|e| format!("访问 WebView 失败: {e}"))?;

        let pdf_data = tokio::time::timeout(Duration::from_secs(30), rx)
            .await
            .map_err(|_| "PDF 生成超时（30s）".to_string())?
            .map_err(|_| "PDF 生成通道已关闭".to_string())??;

        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建输出目录失败: {e}"))?;
        }
        std::fs::write(&out, &pdf_data).map_err(|e| format!("写入 PDF 失败: {e}"))?;
        Ok(())
    }
}

#[cfg(windows)]
mod win {
    use super::*;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::sync::{Arc, Mutex};
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2_7;
    use webview2_com::PrintToPdfCompletedHandler;
    use windows::core::{BOOL, HRESULT, Interface};

    pub async fn print_pdf<R: Runtime>(
        window: &tauri::WebviewWindow<R>,
        out_path: &Path,
    ) -> Result<(), String> {
        let wide: Vec<u16> = OsStr::new(out_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let out = out_path.to_path_buf();

        let (tx, rx) = tokio::sync::oneshot::channel::<Result<(), String>>();
        let tx = Arc::new(Mutex::new(Some(tx)));

        window
            .with_webview({
                let tx = Arc::clone(&tx);
                move |wv| unsafe {
                    let send_err = |msg: String| {
                        if let Some(sender) = tx.lock().ok().and_then(|mut g| g.take()) {
                            let _ = sender.send(Err(msg));
                        }
                    };

                    let controller = wv.controller();
                    let base = match controller.CoreWebView2() {
                        Ok(v) => v,
                        Err(e) => {
                            send_err(format!("获取 WebView2 失败: {e}"));
                            return;
                        }
                    };
                    let webview = match base.cast::<ICoreWebView2_7>() {
                        Ok(v) => v,
                        Err(_) => {
                            send_err("当前 WebView2 不支持 PrintToPdf".to_string());
                            return;
                        }
                    };

                    let handler = PrintToPdfCompletedHandler::create(Box::new(
                        move |error_code: HRESULT, _ok: BOOL| {
                            let result = if error_code.is_ok() {
                                Ok(())
                            } else {
                                Err(format!("PDF 生成失败: {error_code}"))
                            };
                            if let Some(sender) = tx.lock().ok().and_then(|mut g| g.take()) {
                                let _ = sender.send(result);
                            }
                            Ok(())
                        },
                    ));

                    if let Err(e) = webview.PrintToPdf(
                        windows::core::PCWSTR::from_raw(wide.as_ptr()),
                        None,
                        &handler,
                    ) {
                        send_err(format!("PrintToPdf 调用失败: {e}"));
                    }
                }
            })
            .map_err(|e| format!("访问 WebView 失败: {e}"))?;

        tokio::time::timeout(Duration::from_secs(30), rx)
            .await
            .map_err(|_| "PDF 生成超时（30s）".to_string())?
            .map_err(|_| "PDF 生成通道已关闭".to_string())??;

        if !out.exists() {
            return Err("PDF 文件未生成".to_string());
        }
        Ok(())
    }
}

pub fn resolve_html_entry(path: &Path) -> Result<PathBuf, String> {
    if path.is_dir() {
        let index = path.join("index.html");
        if index.is_file() {
            return Ok(index);
        }
        return Err(format!("目录 {} 下未找到 index.html", path.display()));
    }
    if path.extension().and_then(|e| e.to_str()) == Some("html") {
        return Ok(path.to_path_buf());
    }
    Err(format!(
        "不支持的输入类型: {}（需要 .html 文件或含 index.html 的目录）",
        path.display()
    ))
}
