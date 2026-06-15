use std::path::Path;
use tauri::{AppHandle, Manager, Runtime};

pub fn allow_project_root<R: Runtime>(app: &AppHandle<R>, root_path: &str) {
    let path = Path::new(root_path);
    if path.is_absolute() {
        let _ = app.asset_protocol_scope().allow_directory(path, true);
    }
    if let Ok(canonical) = path.canonicalize() {
        let _ = app.asset_protocol_scope().allow_directory(&canonical, true);
    }
}

pub fn allow_project_roots<R: Runtime>(
    app: &AppHandle<R>,
    roots: impl IntoIterator<Item = impl AsRef<str>>,
) {
    for root in roots {
        allow_project_root(app, root.as_ref());
    }
}
