use crate::tools::ToolError;
use std::fs::{self, File};
use std::io::{copy, Read};
use std::path::{Path, PathBuf};
use zip::read::ZipArchive;
use zip::write::SimpleFileOptions;

pub struct UnpackReport {
    pub parts: usize,
}

pub fn unpack(src: &Path, out_dir: &Path, merge_runs: bool) -> Result<UnpackReport, ToolError> {
    if let Some(ext) = src.extension().and_then(|e| e.to_str()) {
        if crate::tools::office::legacy_target_extension(ext).is_some() {
            return Err(ToolError::InvalidArgs(
                "旧格式不支持解包编辑，请先使用 office_convert 转为 .docx/.pptx/.xlsx".into(),
            ));
        }
    }
    if out_dir.exists() {
        fs::remove_dir_all(out_dir)
            .map_err(|e| ToolError::Execution(format!("remove {}: {e}", out_dir.display())))?;
    }
    fs::create_dir_all(out_dir)
        .map_err(|e| ToolError::Execution(format!("mkdir {}: {e}", out_dir.display())))?;

    let file = File::open(src)
        .map_err(|e| ToolError::Execution(format!("open {}: {e}", src.display())))?;
    let mut archive = ZipArchive::new(file).map_err(|e| ToolError::Execution(e.to_string()))?;
    let parts = archive.len();
    for i in 0..parts {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| ToolError::Execution(e.to_string()))?;
        let name = entry.name().to_string();
        if name.ends_with('/') {
            // Directory entry: nothing to write.
            continue;
        }
        let out_path = out_dir.join(&name);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ToolError::Execution(format!("mkdir {}: {e}", parent.display())))?;
        }
        if is_xml_part(&name) {
            let mut raw = Vec::new();
            entry
                .read_to_end(&mut raw)
                .map_err(|e| ToolError::Execution(format!("read entry {name}: {e}")))?;
            let text = String::from_utf8_lossy(&raw);
            let merged = if merge_runs && name.contains("document.xml") {
                merge_runs_simple(&text)
            } else {
                text.into_owned()
            };
            fs::write(&out_path, merged)
                .map_err(|e| ToolError::Execution(format!("write {}: {e}", out_path.display())))?;
        } else {
            let mut out_file = File::create(&out_path)
                .map_err(|e| ToolError::Execution(format!("create {}: {e}", out_path.display())))?;
            copy(&mut entry, &mut out_file)
                .map_err(|e| ToolError::Execution(format!("copy {}: {e}", out_path.display())))?;
        }
    }
    Ok(UnpackReport { parts })
}

fn is_xml_part(name: &str) -> bool {
    name.ends_with(".xml") || name.ends_with(".rels")
}

fn merge_runs_simple(xml: &str) -> String {
    // MVP: preserve content; full merge_runs.py port can refine later.
    xml.replace('\u{2019}', "&#x2019;")
        .replace('\u{2018}', "&#x2018;")
        .replace('\u{201c}', "&#x201C;")
        .replace('\u{201d}', "&#x201D;")
}

pub fn repack_from_dir(dir: &Path, out: &Path) -> Result<(), ToolError> {
    let mut files = Vec::new();
    collect_files(dir, dir, &mut files)?;
    files.sort_by(|a, b| {
        let a_ct = a.1.ends_with("[Content_Types].xml");
        let b_ct = b.1.ends_with("[Content_Types].xml");
        b_ct.cmp(&a_ct).then_with(|| a.1.cmp(&b.1))
    });
    let out_file = File::create(out).map_err(|e| ToolError::Execution(e.to_string()))?;
    let mut zip = zip::ZipWriter::new(out_file);
    let options = SimpleFileOptions::default();
    for (path, name) in files {
        zip.start_file(name, options)
            .map_err(|e| ToolError::Execution(e.to_string()))?;
        let mut input = File::open(&path).map_err(|e| ToolError::Execution(e.to_string()))?;
        copy(&mut input, &mut zip).map_err(|e| ToolError::Execution(e.to_string()))?;
    }
    zip.finish()
        .map_err(|e| ToolError::Execution(e.to_string()))?;
    Ok(())
}

fn collect_files(
    base: &Path,
    current: &Path,
    out: &mut Vec<(PathBuf, String)>,
) -> Result<(), ToolError> {
    for entry in fs::read_dir(current).map_err(|e| ToolError::Execution(e.to_string()))? {
        let entry = entry.map_err(|e| ToolError::Execution(e.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(base, &path, out)?;
        } else {
            let rel = path
                .strip_prefix(base)
                .map_err(|e| ToolError::Execution(e.to_string()))?
                .to_string_lossy()
                .replace('\\', "/");
            out.push((path, rel));
        }
    }
    Ok(())
}
