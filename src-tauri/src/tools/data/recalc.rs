use crate::tools::ToolError;
use ironcalc::import::load_from_xlsx;
use serde_json::{json, Value};
use std::path::Path;

const HARD_ERRORS: &[&str] = &["#REF!", "#DIV/0!", "#VALUE!", "#N/A", "#NUM!", "#NULL!"];

pub fn recalc(path: &Path) -> Result<Value, ToolError> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| recalc_inner(path))) {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(e),
        Err(_) => Ok(json!({
            "errors": [],
            "warnings": [{ "message": "IronCalc could not evaluate this workbook; formula check skipped" }]
        })),
    }
}

fn recalc_inner(path: &Path) -> Result<Value, ToolError> {
    let mut model = load_from_xlsx(
        path.to_str()
            .ok_or_else(|| ToolError::InvalidArgs("invalid path".into()))?,
        "en",
        "UTC",
        "en",
    )
    .map_err(|e| ToolError::Execution(e.to_string()))?;
    model.evaluate();

    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    for cell in model.get_all_cells() {
        let text = model
            .get_formatted_cell_value(cell.index, cell.row, cell.column)
            .map_err(ToolError::Execution)?;
        if HARD_ERRORS.iter().any(|e| text.contains(e)) {
            errors.push(json!({
                "sheet": cell.index,
                "row": cell.row,
                "col": cell.column,
                "value": text
            }));
        } else if text.contains("#NAME?") {
            warnings.push(json!({
                "sheet": cell.index,
                "row": cell.row,
                "col": cell.column,
                "value": text
            }));
        }
    }
    Ok(json!({ "errors": errors, "warnings": warnings }))
}
