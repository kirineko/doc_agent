use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const PDFIUM_RELEASE: &str = "7763";
const NOTO_CJK_REPO: &str = "https://github.com/notofonts/noto-cjk/raw/refs/heads/main";

const NOTO_FONTS: &[(&str, &str)] = &[
    (
        "NotoSansSC-Regular.otf",
        "Sans/SubsetOTF/SC/NotoSansSC-Regular.otf",
    ),
    (
        "NotoSansSC-Bold.otf",
        "Sans/SubsetOTF/SC/NotoSansSC-Bold.otf",
    ),
    (
        "NotoSerifSC-Regular.otf",
        "Serif/SubsetOTF/SC/NotoSerifSC-Regular.otf",
    ),
    (
        "NotoSerifSC-Bold.otf",
        "Serif/SubsetOTF/SC/NotoSerifSC-Bold.otf",
    ),
];

fn main() {
    ensure_pdfium();
    ensure_noto_fonts();
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

    let vendor_lib = vendor_dir.join(lib_name);
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
    fs::copy(&vendor_lib, target_dir.join(lib_name)).ok();

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

fn ensure_noto_fonts() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let fonts_dir = manifest_dir.join("fonts");
    fs::create_dir_all(&fonts_dir).ok();

    for (file_name, rel_path) in NOTO_FONTS {
        let dest = fonts_dir.join(file_name);
        if dest.exists() {
            continue;
        }
        let url = format!("{NOTO_CJK_REPO}/{rel_path}");
        if let Err(err) = download_file(&url, &dest) {
            println!("cargo:warning=failed to download Noto font {file_name}: {err}");
        }
    }

    println!("cargo:rerun-if-changed=build.rs");
}

fn download_file(url: &str, dest: &Path) -> Result<(), String> {
    let mut response = ureq::get(url)
        .call()
        .map_err(|e| format!("download request failed: {e}"))?;
    let bytes = response
        .body_mut()
        .with_config()
        .limit(50 * 1024 * 1024)
        .read_to_vec()
        .map_err(|e| format!("download read failed: {e}"))?;
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(dest, bytes).map_err(|e| e.to_string())?;
    Ok(())
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
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(archive_bytes.as_slice()));
    for entry in archive
        .entries()
        .map_err(|e| format!("extract failed: {e}"))?
    {
        let mut entry = entry.map_err(|e| format!("extract failed: {e}"))?;
        let path = entry.path().map_err(|e| format!("extract failed: {e}"))?;
        let top_level = path
            .components()
            .next()
            .and_then(|component| component.as_os_str().to_str());
        if matches!(top_level, Some("lib" | "bin")) {
            entry
                .unpack_in(&temp_dir)
                .map_err(|e| format!("extract failed: {e}"))?;
        }
    }

    let extracted = ["lib", "bin"]
        .iter()
        .map(|dir| temp_dir.join(dir).join(lib_name))
        .find(|path| path.exists())
        .ok_or_else(|| format!("missing {lib_name} in PDFium archive"))?;

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::copy(&extracted, dest).map_err(|e| e.to_string())?;
    Ok(())
}
