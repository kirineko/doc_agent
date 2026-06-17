use super::scan::{
    attr_local, count_effective_child, for_each_element, root_element, ElementStack, ScanEvent,
};
use crate::tools::ooxml::validate::error::RuleViolation;

const SHEET_STATES: &[&str] = &["visible", "hidden", "veryHidden"];

fn is_mce_wrapper(name: &str) -> bool {
    matches!(name, "AlternateContent" | "Choice" | "Fallback")
}

pub fn validate_workbook(xml: &str) -> Vec<RuleViolation> {
    let mut v = Vec::new();
    match root_element(xml) {
        Ok((name, _)) if name == "workbook" => {}
        Ok((name, _)) => {
            v.push(RuleViolation::new(
                "sml.wb.01",
                "ISO-IEC29500-4_2016/sml.xsd#CT_Workbook",
                format!("root must be workbook, got {name}"),
                None,
            ));
            return v;
        }
        Err(e) => {
            v.push(RuleViolation::new(
                "sml.wb.01",
                "ISO-IEC29500-4_2016/sml.xsd#CT_Workbook",
                format!("parse error: {e}"),
                None,
            ));
            return v;
        }
    }

    let sheets_count = count_effective_child(xml, "workbook", "sheets").unwrap_or(0);
    if sheets_count != 1 {
        v.push(RuleViolation::new(
            "sml.wb.02",
            "ISO-IEC29500-4_2016/sml.xsd#CT_Workbook",
            format!("workbook must contain exactly one sheets element, found {sheets_count}"),
            None,
        ));
    }

    let mut stack = ElementStack::default();
    let mut sheet_count = 0usize;
    let _ = for_each_element(xml, |ev| match ev {
        ScanEvent::Start(name, e, line) => {
            if stack.current() == Some("sheets") && name == "sheet" {
                sheet_count += 1;
                for attr in ["name", "sheetId"] {
                    if attr_local(e, attr).is_none() {
                        v.push(RuleViolation::new(
                            "sml.wb.03",
                            "ISO-IEC29500-4_2016/sml.xsd#CT_Sheet",
                            format!("sheet missing @{attr}"),
                            Some(line),
                        ));
                    }
                }
                let rid = attr_local(e, "id");
                if rid.is_none() {
                    v.push(RuleViolation::new(
                        "sml.wb.03",
                        "ISO-IEC29500-4_2016/sml.xsd#CT_Sheet",
                        "sheet missing relationship id (r:id or id)",
                        Some(line),
                    ));
                }
                if let Some(state) = attr_local(e, "state") {
                    if !SHEET_STATES.contains(&state.as_str()) {
                        v.push(RuleViolation::new(
                            "sml.wb.04",
                            "ISO-IEC29500-4_2016/sml.xsd#ST_SheetState",
                            format!("invalid sheet state '{state}'"),
                            Some(line),
                        ));
                    }
                }
            }
            stack.on_start(name);
        }
        ScanEvent::End(name) => stack.on_end(name),
    });

    if sheets_count == 1 && sheet_count == 0 {
        v.push(RuleViolation::new(
            "sml.wb.03",
            "ISO-IEC29500-4_2016/sml.xsd#CT_Sheets",
            "sheets must contain at least one sheet",
            None,
        ));
    }
    v
}

pub fn validate_worksheet(xml: &str) -> Vec<RuleViolation> {
    let mut v = Vec::new();
    match root_element(xml) {
        Ok((name, _)) if name == "worksheet" => {}
        Ok((name, _)) => {
            v.push(RuleViolation::new(
                "sml.ws.01",
                "ISO-IEC29500-4_2016/sml.xsd#CT_Worksheet",
                format!("root must be worksheet, got {name}"),
                None,
            ));
            return v;
        }
        Err(e) => {
            v.push(RuleViolation::new(
                "sml.ws.01",
                "ISO-IEC29500-4_2016/sml.xsd#CT_Worksheet",
                format!("parse error: {e}"),
                None,
            ));
            return v;
        }
    }

    let sheet_data_count = count_effective_child(xml, "worksheet", "sheetData").unwrap_or(0);
    if sheet_data_count != 1 {
        v.push(RuleViolation::new(
            "sml.ws.02",
            "ISO-IEC29500-4_2016/sml.xsd#CT_Worksheet",
            format!("worksheet must contain exactly one sheetData, found {sheet_data_count}"),
            None,
        ));
    }

    let mut stack = ElementStack::default();
    let _ = for_each_element(xml, |ev| match ev {
        ScanEvent::Start(name, _, line) => {
            if stack.current() == Some("sheetData") && name != "row" && !is_mce_wrapper(name) {
                v.push(RuleViolation::new(
                    "sml.ws.03",
                    "ISO-IEC29500-4_2016/sml.xsd#CT_SheetData",
                    format!("sheetData must only contain row, got '{name}'"),
                    Some(line),
                ));
            }
            if stack.current() == Some("row")
                && name != "c"
                && name != "extLst"
                && !is_mce_wrapper(name)
            {
                v.push(RuleViolation::new(
                    "sml.ws.04",
                    "ISO-IEC29500-4_2016/sml.xsd#CT_Row",
                    format!("row must only contain c or extLst, got '{name}'"),
                    Some(line),
                ));
            }
            stack.on_start(name);
        }
        ScanEvent::End(name) => stack.on_end(name),
    });
    v
}
