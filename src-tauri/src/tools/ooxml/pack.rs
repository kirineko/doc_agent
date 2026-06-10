use super::unpack::repack_from_dir;
use super::validate;
use crate::tools::ToolError;
use std::fs;
use std::path::Path;

pub fn pack(dir: &Path, out: &Path, original: Option<&Path>) -> Result<(), ToolError> {
    auto_repair(dir)?;
    validate::validate_dir(dir, original)?;
    if out.exists() {
        fs::remove_file(out).ok();
    }
    repack_from_dir(dir, out)?;
    validate::roundtrip_check(out)?;
    Ok(())
}

fn auto_repair(_dir: &Path) -> Result<(), ToolError> {
    Ok(())
}
