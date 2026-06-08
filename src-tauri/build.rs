use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const PDFIUM_RELEASE: &str = "7763";

fn main() {
    ensure_pdfium();
    tauri_build::build();
}

fn ensure_pdfium() {
    let target = match env::var("TARGET") {
        Ok(value) => value,
        Err(_) => return,
    };

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let vendor_dir = manifest_dir.join("pdfium");
    fs::create_dir_all(&vendor_dir).ok();

    let Some((url, lib_name)) = pdfium_asset(&target) else {
        println!("cargo:warning=unsupported target for bundled PDFium: {target}");
        return;
    };

    let vendor_lib = vendor_dir.join(&lib_name);
    if !vendor_lib.exists() {
        if let Err(err) = download_pdfium(url, &vendor_lib, lib_name) {
            println!("cargo:warning=failed to download PDFium: {err}");
            return;
        }
    }

    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let target_dir = env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("target"))
        .join(&target)
        .join(&profile);
    fs::create_dir_all(&target_dir).ok();
    fs::copy(&vendor_lib, target_dir.join(&lib_name)).ok();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-env=PDFIUM_LIB_NAME={lib_name}");
}

fn pdfium_asset(target: &str) -> Option<(&'static str, &'static str)> {
    Some(match target {
        "aarch64-apple-darwin" => (
            "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F7763/pdfium-mac-arm64.tgz",
            "libpdfium.dylib",
        ),
        "x86_64-apple-darwin" => (
            "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F7763/pdfium-mac-x64.tgz",
            "libpdfium.dylib",
        ),
        "x86_64-pc-windows-msvc" => (
            "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F7763/pdfium-win-x64.tgz",
            "pdfium.dll",
        ),
        "aarch64-pc-windows-msvc" => (
            "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F7763/pdfium-win-arm64.tgz",
            "pdfium.dll",
        ),
        "x86_64-unknown-linux-gnu" => (
            "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F7763/pdfium-linux-x64.tgz",
            "libpdfium.so",
        ),
        "aarch64-unknown-linux-gnu" => (
            "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F7763/pdfium-linux-arm64.tgz",
            "libpdfium.so",
        ),
        _ => return None,
    })
}

fn download_pdfium(url: &str, dest: &Path, lib_name: &str) -> Result<(), String> {
    let response = ureq::get(url)
        .call()
        .map_err(|e| format!("download request failed: {e}"))?;
    let archive_bytes = response
        .into_body()
        .read_to_vec()
        .map_err(|e| format!("download read failed: {e}"))?;

    let temp_dir = env::temp_dir().join(format!("doc-agent-pdfium-{PDFIUM_RELEASE}"));
    fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;
    if temp_dir.join("lib").exists() {
        fs::remove_dir_all(temp_dir.join("lib")).map_err(|e| e.to_string())?;
    }

    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(archive_bytes.as_slice()));
    for entry in archive
        .entries()
        .map_err(|e| format!("extract failed: {e}"))?
    {
        let mut entry = entry.map_err(|e| format!("extract failed: {e}"))?;
        let path = entry.path().map_err(|e| format!("extract failed: {e}"))?;
        if path.starts_with("lib") {
            entry.unpack_in(&temp_dir).map_err(|e| format!("extract failed: {e}"))?;
        }
    }

    let extracted = temp_dir.join("lib").join(lib_name);
    if !extracted.exists() {
        return Err(format!("missing {lib_name} in PDFium archive"));
    }

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::copy(&extracted, dest).map_err(|e| e.to_string())?;
    Ok(())
}
