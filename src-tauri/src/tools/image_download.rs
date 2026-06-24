use crate::tools::{ToolContext, ToolError};
use futures_util::StreamExt;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::io::Cursor;
use std::net::IpAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const MAX_URLS: usize = 20;
const MAX_BYTES: usize = 15 * 1024 * 1024;
const CONCURRENCY: usize = 5;
const TIMEOUT_SECS: u64 = 30;
const DEFAULT_DIR: &str = "images";
/// 重定向跳数上限（与 reqwest 默认一致）。自定义 redirect Policy 覆盖了
/// reqwest 内置上限，须自行封顶，否则重定向环会一直跟随到请求超时。
const MAX_REDIRECTS: usize = 10;

pub fn tool() -> crate::tools::ToolSpec {
    crate::tools::ToolSpec {
        name: "image_download",
        description: "Download 1-20 public http(s) image URLs to a project folder and return local relative paths. \
            Use this to localize remote images before inserting them into documents: skill_run / typst_to_pdf / html_to_pdf only reference local files (the skill_run runtime has no network). \
            Independent of docx/pptx/pdf generation — it only fetches files. \
            Returns downloaded[] ({url, path, bytes, width, height, format}) and failed[] ({url, error}); reference the returned path values when building documents.",
        parameters: json!({
            "type": "object",
            "properties": {
                "urls": {
                    "type": "array",
                    "items": { "type": "string" },
                    "minItems": 1,
                    "maxItems": MAX_URLS,
                    "description": "Public http(s) image URLs to download (1-20)"
                },
                "dir": {
                    "type": "string",
                    "default": DEFAULT_DIR,
                    "description": "Project-relative output folder (default 'images'); created if missing"
                }
            },
            "required": ["urls"],
            "additionalProperties": false
        }),
        handler: |_ctx, _args| Err(ToolError::NotImplemented),
    }
}

struct Saved {
    url: String,
    path: String,
    bytes: u64,
    width: u32,
    height: u32,
    format: String,
}

struct Failed {
    url: String,
    error: String,
}

pub async fn handler(ctx: &ToolContext<'_>, args: Value) -> Result<Value, ToolError> {
    let urls = parse_urls(&args)?;

    let dir_norm = normalize_output_dir(args.get("dir").and_then(|v| v.as_str()))?;

    let out_dir = ctx
        .sandbox
        .resolve_for_write(&dir_norm)
        .map_err(ToolError::Sandbox)?;
    // resolve_for_write 对尚不存在的叶子仅做词法前缀校验、不跟随符号链接。
    // 若祖先目录是指向项目外的符号链接（如 images -> /tmp/outside），create_dir_all
    // 会沿链接在沙箱外建目录、随后写出文件（逃逸）。写盘前先校验最近的已存在祖先
    // canonicalize 后仍位于 root 内。
    ensure_output_dir_within_sandbox(ctx.sandbox.root(), &out_dir)?;
    std::fs::create_dir_all(&out_dir).map_err(|e| ToolError::Execution(e.to_string()))?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(TIMEOUT_SECS))
        .user_agent(concat!("doc-agent/", env!("CARGO_PKG_VERSION")))
        // 重定向 SSRF 防护 + 跳数封顶：自定义 Policy 覆盖了 reqwest 内置的 10 跳
        // 上限，须自行封顶，否则重定向环会一直跟随到超时。每次跳转先对目标 URL
        // 重新跑 validate_url（私网/环回拒绝），再校验跳数。
        .redirect(reqwest::redirect::Policy::custom(
            |attempt| match redirect_decision(attempt.previous().len(), attempt.url().as_str()) {
                RedirectDecision::Follow => attempt.follow(),
                RedirectDecision::Block => attempt.stop(),
                RedirectDecision::TooMany => {
                    attempt.error(format!("too many redirects (>{MAX_REDIRECTS})"))
                }
            },
        ))
        .build()
        .map_err(|e| ToolError::Execution(format!("failed to init HTTP client: {e}")))?;

    // 每个并发任务自持其数据（owned clones / Arc），使 future 满足 Send（IPC 命令要求）。
    let dir_label = dir_norm.clone();
    let out_dir = Arc::new(out_dir);
    let dir_norm = Arc::new(dir_norm);
    let used: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

    let mut results: Vec<(usize, Result<Saved, Failed>)> =
        futures_util::stream::iter(urls.into_iter().enumerate())
            .map(move |(idx, url)| {
                let client = client.clone();
                let out_dir = out_dir.clone();
                let dir_norm = dir_norm.clone();
                let used = used.clone();
                async move {
                    let outcome = fetch_one(&client, &url, &out_dir, &dir_norm, &used).await;
                    (idx, outcome)
                }
            })
            .buffer_unordered(CONCURRENCY)
            .collect()
            .await;
    results.sort_by_key(|(idx, _)| *idx);

    let mut downloaded = Vec::new();
    let mut failed = Vec::new();
    for (_, outcome) in results {
        match outcome {
            Ok(s) => downloaded.push(json!({
                "url": s.url,
                "path": s.path,
                "bytes": s.bytes,
                "width": s.width,
                "height": s.height,
                "format": s.format,
            })),
            Err(f) => failed.push(json!({ "url": f.url, "error": f.error })),
        }
    }

    Ok(json!({
        "dir": dir_label,
        "downloaded": downloaded,
        "failed": failed,
        "count": downloaded.len(),
    }))
}

fn parse_urls(args: &Value) -> Result<Vec<String>, ToolError> {
    let items = args
        .get("urls")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ToolError::InvalidArgs("urls required (array of strings)".into()))?;
    if items.is_empty() {
        return Err(ToolError::InvalidArgs("urls must not be empty".into()));
    }
    if items.len() > MAX_URLS {
        return Err(ToolError::InvalidArgs(format!(
            "urls must contain at most {MAX_URLS} items"
        )));
    }
    let mut urls = Vec::with_capacity(items.len());
    for item in items {
        let url = item
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgs("each url must be a string".into()))?
            .trim();
        if url.is_empty() {
            return Err(ToolError::InvalidArgs("url must not be empty".into()));
        }
        urls.push(url.to_string());
    }
    Ok(urls)
}

/// 规范化输出目录：handler 与 io_plan 共用，保证两者对 `dir` 的处理一致。
///
/// - trim 后空/缺省 → `DEFAULT_DIR`（`images`）；
/// - 反斜杠归一为 `/`，去尾 `/`，折叠冗余 `.` 段（`./.cache` → `.cache`、
///   `foo/./bar` → `foo/bar`），避免 `./` 前缀绕过 `.cache`/项目根检查；
/// - 拒绝绝对路径、含 `..` 的越界路径；
/// - 拒绝 `.`（项目根，会申请过粗的 SubtreeWrite 且图片散落根目录）；
/// - 拒绝 `.cache` 首段：该目录会被 `changed_paths` 静默过滤，文件落盘但
///   UI / `@` / 产物面板不可见，与默认 `images/` 的"产物可见"语义冲突。
pub fn normalize_output_dir(dir: Option<&str>) -> Result<String, ToolError> {
    let raw = dir
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_DIR);
    let norm = raw.replace('\\', "/");
    // 折叠 `.` 段（保留 `..` 以便后续越界判定）。`foo/./bar` → `foo/bar`，
    // `./.cache` → `.cache`，使首段检查能正确命中 `.cache`。
    let folded: Vec<&str> = norm.split('/').filter(|seg| *seg != ".").collect();
    let norm = folded.join("/");
    let norm = norm.trim_end_matches('/').to_string();
    if Path::new(&norm).is_absolute() || norm.split('/').any(|c| c == "..") {
        return Err(ToolError::InvalidArgs(
            "dir must be a project-relative folder without '..'".into(),
        ));
    }
    if norm.is_empty() {
        return Err(ToolError::InvalidArgs(
            "dir must not be the project root; use a subfolder (default 'images')".into(),
        ));
    }
    let first = norm.split('/').next().unwrap_or("");
    if first == crate::core::cache_paths::CACHE_ROOT {
        return Err(ToolError::InvalidArgs(
            "dir must not be under .cache/ (those files are hidden from the workspace)".into(),
        ));
    }
    Ok(norm)
}

/// 校验输出目录不会经「符号链接祖先」逃逸沙箱。
/// 从 out_dir 向上找最近的已存在祖先，canonicalize 后确认仍在 root 内；
/// 任一祖先解析到 root 外即拒绝（root 自身已是 canonical）。
fn ensure_output_dir_within_sandbox(root: &Path, out_dir: &Path) -> Result<(), ToolError> {
    let mut ancestor = out_dir;
    loop {
        // symlink_metadata 不跟随：断链/有效符号链接都视为“存在”，再由 canonicalize
        // 判定真实目标是否越界（断链祖先 canonicalize 失败 → Execution 错误，仍不写盘）。
        if ancestor.symlink_metadata().is_ok() {
            let canonical = ancestor
                .canonicalize()
                .map_err(|e| ToolError::Execution(e.to_string()))?;
            if !canonical.starts_with(root) {
                return Err(ToolError::Sandbox(
                    crate::core::sandbox::SandboxError::EscapesSandbox,
                ));
            }
            return Ok(());
        }
        match ancestor.parent() {
            Some(parent) => ancestor = parent,
            None => return Ok(()),
        }
    }
}

async fn fetch_one(
    client: &reqwest::Client,
    url: &str,
    out_dir: &Path,
    dir_norm: &str,
    used: &Mutex<HashSet<String>>,
) -> Result<Saved, Failed> {
    let fail = |error: String| Failed {
        url: url.to_string(),
        error,
    };

    if let Err(reason) = validate_url(url) {
        return Err(fail(reason));
    }

    let bytes = fetch_bytes(client, url).await.map_err(fail)?;
    let (format, width, height) =
        detect_image(&bytes).ok_or_else(|| fail("not a supported image".into()))?;

    let stem = filename_stem_from_url(url);
    let name = reserve_name(used, out_dir, &stem, &format);
    let abs = out_dir.join(&name);
    // create_new：仅当文件不存在时创建。与 reserve_name 的 symlink_metadata 探测
    // 叠加，堵住 reserve→write 之间的 TOCTOU，也确保不覆盖既有文件/符号链接。
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&abs)
        .map_err(|e| fail(format!("write failed: {e}")))?;
    f.write_all(&bytes)
        .map_err(|e| fail(format!("write failed: {e}")))?;
    f.flush().map_err(|e| fail(format!("write failed: {e}")))?;

    Ok(Saved {
        url: url.to_string(),
        path: join_rel(dir_norm, &name),
        bytes: bytes.len() as u64,
        width,
        height,
        format,
    })
}

async fn fetch_bytes(client: &reqwest::Client, url: &str) -> Result<Vec<u8>, String> {
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status().as_u16()));
    }
    let mut stream = resp.bytes_stream();
    let mut buf = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("stream error: {e}"))?;
        if buf.len() + chunk.len() > MAX_BYTES {
            return Err(format!(
                "exceeds {} MiB size limit",
                MAX_BYTES / 1024 / 1024
            ));
        }
        buf.extend_from_slice(&chunk);
    }
    if buf.is_empty() {
        return Err("empty response body".into());
    }
    Ok(buf)
}

/// 按真实字节判定图片格式与尺寸；非受支持图片返回 None（防 HTML/JSON 错误页伪装）。
fn detect_image(bytes: &[u8]) -> Option<(String, u32, u32)> {
    let reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let ext = match reader.format()? {
        image::ImageFormat::Png => "png",
        image::ImageFormat::Jpeg => "jpg",
        image::ImageFormat::Gif => "gif",
        image::ImageFormat::WebP => "webp",
        image::ImageFormat::Bmp => "bmp",
        image::ImageFormat::Tiff => "tiff",
        _ => return None,
    };
    let (width, height) = reader.into_dimensions().ok()?;
    Some((ext.to_string(), width, height))
}

#[derive(Debug, PartialEq, Eq)]
enum RedirectDecision {
    Follow,
    Block,
    TooMany,
}

/// 重定向决策：越界目标优先拒绝（安全第一），其次按跳数封顶。
/// `hops` 为已发生的跳转次数（`Attempt::previous().len()`）。
fn redirect_decision(hops: usize, target: &str) -> RedirectDecision {
    if validate_url(target).is_err() {
        RedirectDecision::Block
    } else if hops >= MAX_REDIRECTS {
        RedirectDecision::TooMany
    } else {
        RedirectDecision::Follow
    }
}

/// 仅 http/https；尽力而为拒绝环回/私网/链路本地/未指定/ULA 与 localhost、*.local。
fn validate_url(raw: &str) -> Result<(), String> {
    let url = reqwest::Url::parse(raw).map_err(|e| format!("invalid URL: {e}"))?;
    match url.scheme() {
        "http" | "https" => {}
        other => return Err(format!("unsupported scheme: {other} (only http/https)")),
    }
    let host = url.host_str().ok_or_else(|| "missing host".to_string())?;
    if is_blocked_host(host) {
        return Err(format!("blocked non-public host: {host}"));
    }
    Ok(())
}

fn is_blocked_host(host: &str) -> bool {
    let literal = host.trim_start_matches('[').trim_end_matches(']');
    if let Ok(ip) = literal.parse::<IpAddr>() {
        return is_blocked_ip(ip);
    }
    // 尾点 host（如 localhost. / printer.local.）在常见解析器里仍解析为
    // 同一 localhost/.local 目标，须先剥尾点再做域名规则匹配。
    let lower = host.trim_end_matches('.').to_ascii_lowercase();
    lower == "localhost" || lower.ends_with(".localhost") || lower.ends_with(".local")
}

fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
        }
        IpAddr::V6(v6) => {
            if v6.is_loopback() || v6.is_unspecified() {
                return true;
            }
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_blocked_ip(IpAddr::V4(v4));
            }
            let seg = v6.segments();
            // fc00::/7 (ULA) 与 fe80::/10 (link-local)
            (seg[0] & 0xfe00) == 0xfc00 || (seg[0] & 0xffc0) == 0xfe80
        }
    }
}

fn filename_stem_from_url(raw: &str) -> String {
    let last = reqwest::Url::parse(raw)
        .ok()
        .and_then(|u| {
            u.path_segments()
                .and_then(|mut segs| segs.next_back().map(|s| s.to_string()))
        })
        .unwrap_or_default();
    let stem = Path::new(&last)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let cleaned = sanitize(stem);
    if cleaned.is_empty() {
        "image".to_string()
    } else {
        cleaned
    }
}

fn sanitize(name: &str) -> String {
    let mapped: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    mapped.trim_matches('_').chars().take(64).collect()
}

fn reserve_name(used: &Mutex<HashSet<String>>, out_dir: &Path, stem: &str, ext: &str) -> String {
    let mut guard = used.lock().expect("filename mutex poisoned");
    let mut candidate = format!("{stem}.{ext}");
    let mut counter = 1;
    // 用 symlink_metadata（不跟随符号链接）探测占用：既匹配普通文件，也匹配
    // 断链/有效符号链接。否则 `out_dir/logo.png -> /tmp/x` 这类断链会被
    // exists()（跟随）判为可用，随后 fs::write 沿链接写出项目外（沙箱逃逸）。
    while guard.contains(&candidate) || out_dir.join(&candidate).symlink_metadata().is_ok() {
        candidate = format!("{stem}-{counter}.{ext}");
        counter += 1;
    }
    guard.insert(candidate.clone());
    candidate
}

fn join_rel(dir_norm: &str, name: &str) -> String {
    if dir_norm.is_empty() || dir_norm == "." {
        name.to_string()
    } else {
        format!("{dir_norm}/{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::sandbox::Sandbox;
    use tempfile::tempdir;

    #[test]
    fn tool_is_registered_name() {
        assert_eq!(tool().name, "image_download");
    }

    #[test]
    fn blocks_non_http_schemes() {
        assert!(validate_url("file:///etc/passwd").is_err());
        assert!(validate_url("data:image/png;base64,AAAA").is_err());
        assert!(validate_url("ftp://example.com/a.png").is_err());
    }

    #[test]
    fn blocks_private_and_loopback_hosts() {
        assert!(validate_url("http://127.0.0.1/a.png").is_err());
        assert!(validate_url("http://localhost/a.png").is_err());
        assert!(validate_url("http://169.254.169.254/latest/meta-data").is_err());
        assert!(validate_url("http://10.0.0.5/a.png").is_err());
        assert!(validate_url("http://192.168.1.2/a.png").is_err());
        assert!(validate_url("http://[::1]/a.png").is_err());
        assert!(validate_url("http://printer.local/a.png").is_err());
        // 尾点绝对域名仍解析为 localhost/.local，须拒
        assert!(validate_url("http://localhost./a.png").is_err());
        assert!(validate_url("http://printer.local./a.png").is_err());
    }

    #[test]
    fn allows_public_hosts() {
        assert!(validate_url("https://example.com/a.png").is_ok());
        assert!(validate_url("http://93.184.216.34/a.png").is_ok());
    }

    #[test]
    fn blocks_noncanonical_ip_literals() {
        // reqwest::Url 会把非标准 IP 写法归一化，is_blocked_host 据此全部拒绝：
        // 十进制 / 短写 / 十六进制 / 八进制 / 0 / IPv4-mapped IPv6 均不得绕过。
        for raw in [
            "http://2130706433/a.png", // 127.0.0.1 (decimal)
            "http://127.1/a.png",      // 127.0.0.1 (short)
            "http://0x7f000001/a.png", // 127.0.0.1 (hex)
            "http://0/a.png",          // 0.0.0.0
            "http://0.0.0.0/a.png",
            "http://[::ffff:127.0.0.1]/a.png",
            "http://[::ffff:7f00:1]/a.png",
        ] {
            assert!(
                validate_url(raw).is_err(),
                "noncanonical IP literal should be blocked: {raw}"
            );
        }
    }

    #[test]
    fn validate_url_blocks_redirect_targets() {
        // 重定向 SSRF 防护：client 的 redirect Policy 对每次跳转目标重新跑
        // validate_url。这里覆盖典型的恶意重定向目标——它们必须被拒，否则
        // 一个公网 URL 经 302 就能打到云元数据 / 内网。
        assert!(validate_url("http://169.254.169.254/latest/meta-data/iam/").is_err());
        assert!(validate_url("http://10.0.0.1/internal.png").is_err());
        assert!(validate_url("http://[fd00::1]/x.png").is_err()); // ULA
        assert!(validate_url("http://192.168.1.1/admin").is_err());
    }

    #[test]
    fn redirect_decision_blocks_caps_and_follows() {
        // 公网目标且跳数未超 → 跟随
        assert_eq!(
            redirect_decision(0, "https://example.com/a.png"),
            RedirectDecision::Follow
        );
        assert_eq!(
            redirect_decision(MAX_REDIRECTS - 1, "https://example.com/a.png"),
            RedirectDecision::Follow
        );
        // 内网目标即便跳数未超也拒绝（安全优先）
        assert_eq!(
            redirect_decision(0, "http://169.254.169.254/meta"),
            RedirectDecision::Block
        );
        // 公网目标但跳数到顶 → TooMany，避免重定向环跟随到超时
        assert_eq!(
            redirect_decision(MAX_REDIRECTS, "https://example.com/a.png"),
            RedirectDecision::TooMany
        );
        // 越界目标 + 超跳数：越界优先，仍判 Block
        assert_eq!(
            redirect_decision(MAX_REDIRECTS, "http://127.0.0.1/x"),
            RedirectDecision::Block
        );
    }

    #[test]
    fn sanitize_strips_unsafe_chars() {
        assert_eq!(sanitize("a b/c?d=1"), "a_b_c_d_1");
        assert_eq!(sanitize("__weird__"), "weird");
        assert_eq!(sanitize(""), "");
    }

    #[test]
    fn stem_derives_from_url_path() {
        assert_eq!(
            filename_stem_from_url("https://x.com/path/logo.png"),
            "logo"
        );
        assert_eq!(filename_stem_from_url("https://x.com/path/"), "image");
        // 查询串不参与文件名；stem 只取路径末段，扩展名由真实格式决定
        assert_eq!(
            filename_stem_from_url("https://x.com/photo.png?w=100&h=50"),
            "photo"
        );
    }

    #[test]
    fn detect_image_accepts_png_rejects_html() {
        let mut buf = Vec::new();
        let img = image::RgbaImage::new(3, 2);
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        let (format, w, h) = detect_image(&buf).expect("png detected");
        assert_eq!(format, "png");
        assert_eq!((w, h), (3, 2));

        assert!(detect_image(b"<html><body>not found</body></html>").is_none());
    }

    #[test]
    fn detect_image_jpeg_emits_jpg_ext() {
        // JPEG 分支：format 字段为 "jpg"（spec 列举 jpeg，扩展名按惯例用 jpg）
        let mut buf = Vec::new();
        let img = image::GrayImage::new(4, 4);
        image::DynamicImage::ImageLuma8(img)
            .write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Jpeg)
            .unwrap();
        let (format, w, h) = detect_image(&buf).expect("jpeg detected");
        assert_eq!(format, "jpg");
        assert_eq!((w, h), (4, 4));
    }

    #[test]
    fn reserve_name_dedupes() {
        let dir = tempdir().unwrap();
        let used = Mutex::new(HashSet::new());
        let a = reserve_name(&used, dir.path(), "pic", "png");
        let b = reserve_name(&used, dir.path(), "pic", "png");
        assert_eq!(a, "pic.png");
        assert_eq!(b, "pic-1.png");
    }

    #[test]
    fn reserve_name_skips_existing_file_on_disk() {
        // 目录里已存在 pic.png 时，即便 HashSet 为空也要避开，落到 pic-1.png
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("pic.png"), b"x").unwrap();
        let used = Mutex::new(HashSet::new());
        let a = reserve_name(&used, dir.path(), "pic", "png");
        assert_eq!(a, "pic-1.png");
        // 第二次连同磁盘文件与已登记名一起避开
        let b = reserve_name(&used, dir.path(), "pic", "png");
        assert_eq!(b, "pic-2.png");
    }

    #[cfg(unix)]
    #[test]
    fn reserve_name_skips_dangling_symlink() {
        // 沙箱逃逸回归：images/pic.png 若是指向项目外（这里用 tempdir 外的临时
        // 目标）的符号链接，即便目标不存在（断链），也必须被当作占用——否则
        // 后续 fs::write 会沿链接写出项目外。symlink_metadata 不跟随，能识别。
        let dir = tempdir().unwrap();
        let outside = tempdir().unwrap();
        let link = dir.path().join("pic.png");
        std::os::unix::fs::symlink(outside.path().join("stolen.png"), &link).unwrap();
        // 断链确认：exists() 跟随会判为不存在，symlink_metadata 不跟随能识别
        assert!(!link.exists());
        assert!(link.symlink_metadata().is_ok());
        let used = Mutex::new(HashSet::new());
        // 不能复用 pic.png（否则会写到 stolen.png），必须落到 pic-1.png
        let a = reserve_name(&used, dir.path(), "pic", "png");
        assert_eq!(a, "pic-1.png");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn rejects_symlinked_ancestor_dir_escape() {
        // 沙箱逃逸回归（目录级）：项目内 images 是指向项目外目录的符号链接，
        // dir=images/sub（叶子尚不存在）会通过 resolve_for_write 的词法前缀校验，
        // 但 create_dir_all 会沿链接在沙箱外建目录、随后写出文件，必须在写前拒绝。
        let dir = tempdir().unwrap();
        let outside = tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), dir.path().join("images")).unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let ctx = ToolContext::new(&sandbox);
        let err = handler(
            &ctx,
            json!({ "urls": ["https://example.com/logo.png"], "dir": "images/sub" }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, ToolError::Sandbox(_)));
        // 未沿符号链接在项目外创建子目录
        assert!(!outside.path().join("sub").exists());
    }

    #[test]
    fn ensure_output_dir_accepts_legit_nested_dir() {
        // 正常多级目录（无符号链接）：最近的已存在祖先是 root 自身，校验通过
        let dir = tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let out_dir = sandbox.root().join("images").join("sub");
        assert!(ensure_output_dir_within_sandbox(sandbox.root(), &out_dir).is_ok());
    }

    #[test]
    fn join_rel_handles_default_dir() {
        assert_eq!(join_rel("images", "a.png"), "images/a.png");
        assert_eq!(join_rel("", "a.png"), "a.png");
        assert_eq!(join_rel(".", "a.png"), "a.png");
    }

    #[tokio::test]
    async fn rejects_empty_urls() {
        let dir = tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let ctx = ToolContext::new(&sandbox);
        let err = handler(&ctx, json!({ "urls": [] })).await.unwrap_err();
        assert!(matches!(err, ToolError::InvalidArgs(_)));
    }

    #[test]
    fn normalize_output_dir_defaults_and_rejects() {
        // 缺省 / 空白 / 空串 → images
        assert_eq!(normalize_output_dir(None).unwrap(), "images");
        assert_eq!(normalize_output_dir(Some("")).unwrap(), "images");
        assert_eq!(normalize_output_dir(Some("   ")).unwrap(), "images");
        // 反斜杠归一 + 去尾斜杠
        assert_eq!(normalize_output_dir(Some("a\\b\\")).unwrap(), "a/b");
        // 项目根 / .cache / 越界 → 拒绝
        assert!(normalize_output_dir(Some(".")).is_err());
        assert!(normalize_output_dir(Some(".cache")).is_err());
        assert!(normalize_output_dir(Some(".cache/imgs")).is_err());
        assert!(normalize_output_dir(Some("../escape")).is_err());
        assert!(normalize_output_dir(Some("/abs/x")).is_err());
        // `./` 前缀与内嵌 `.` 段不能绕过项目根 .cache 检查（只挡首段 .cache，
        // 因为 changed_paths 只过滤项目根的 .cache；foo/.cache 不在过滤范围）
        assert!(normalize_output_dir(Some("./.cache")).is_err());
        assert!(normalize_output_dir(Some("./.cache/imgs")).is_err());
        assert!(normalize_output_dir(Some(".//cache")).is_err());
        // 冗余 `.` 段被折叠
        assert_eq!(
            normalize_output_dir(Some("images/./x")).unwrap(),
            "images/x"
        );
        assert_eq!(
            normalize_output_dir(Some("./assets/photos")).unwrap(),
            "assets/photos"
        );
        // 正常子目录放行
        assert_eq!(
            normalize_output_dir(Some("assets/photos")).unwrap(),
            "assets/photos"
        );
    }

    #[tokio::test]
    async fn rejects_too_many_urls() {
        let dir = tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let ctx = ToolContext::new(&sandbox);
        let urls: Vec<String> = (0..MAX_URLS + 1)
            .map(|i| format!("https://example.com/{i}.png"))
            .collect();
        let err = handler(&ctx, json!({ "urls": urls })).await.unwrap_err();
        assert!(matches!(err, ToolError::InvalidArgs(_)));
    }

    #[tokio::test]
    async fn rejects_dir_escaping_sandbox() {
        let dir = tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let ctx = ToolContext::new(&sandbox);
        let err = handler(
            &ctx,
            json!({ "urls": ["https://example.com/a.png"], "dir": "../escape" }),
        )
        .await
        .unwrap_err();
        // 越界 dir 在发起任何网络请求前被拒
        assert!(matches!(
            err,
            ToolError::InvalidArgs(_) | ToolError::Sandbox(_)
        ));
    }

    #[tokio::test]
    async fn blocked_urls_report_as_failed_without_network() {
        let dir = tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let ctx = ToolContext::new(&sandbox);
        let out = handler(
            &ctx,
            json!({ "urls": ["file:///etc/passwd", "http://127.0.0.1/a.png"] }),
        )
        .await
        .unwrap();
        assert_eq!(out["count"], json!(0));
        assert_eq!(out["downloaded"].as_array().unwrap().len(), 0);
        assert_eq!(out["failed"].as_array().unwrap().len(), 2);
        // 输出目录已创建于沙箱内
        assert!(dir.path().join("images").is_dir());
    }
}
