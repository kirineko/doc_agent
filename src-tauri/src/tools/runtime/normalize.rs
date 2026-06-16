/// 执行前规范化模型生成的 skill_run 脚本，兼容常见 Node 写法。
pub fn normalize_script(code: &str) -> String {
    let mut out = String::new();
    for line in code.lines() {
        let trimmed = line.trim();
        if let Some(replacement) = rewrite_import_line(trimmed) {
            if !replacement.is_empty() {
                out.push_str(&replacement);
                out.push('\n');
            }
            continue;
        }
        if let Some(replacement) = rewrite_require_line(trimmed) {
            if !replacement.is_empty() {
                out.push_str(&replacement);
                out.push('\n');
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    let mut s = out.trim_end().to_string();
    while s.ends_with("main();") || s.ends_with("await main();") {
        s = s
            .strip_suffix("await main();")
            .or_else(|| s.strip_suffix("main();"))
            .unwrap_or(&s)
            .trim_end()
            .to_string();
    }
    ensure_main_wrapper(&s)
}

/// 模型常省略 main()，却在顶层写 await/return，导致语法错误。
fn ensure_main_wrapper(code: &str) -> String {
    if contains_main_function(code) {
        return code.to_string();
    }
    format!("async function main() {{\n{code}\n}}")
}

fn contains_main_function(code: &str) -> bool {
    code.lines().any(|line| {
        let t = line.trim();
        t.starts_with("async function main")
            || t.starts_with("function main")
            || t.contains("function main(")
    })
}

fn trim_optional_semicolon(line: &str) -> &str {
    line.strip_suffix(';').unwrap_or(line).trim()
}

fn parse_import_from(body: &str) -> Option<(String, String)> {
    let rest = body.strip_prefix("import ")?.trim();
    let from_idx = rest.rfind(" from ")?;
    let bindings = rest[..from_idx].trim().to_string();
    let module_part = rest[from_idx + 6..].trim();
    let module = module_part
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string();
    Some((bindings, module))
}

fn rewrite_import_line(trimmed: &str) -> Option<String> {
    let body = trim_optional_semicolon(trimmed);
    if !body.starts_with("import ") {
        return None;
    }
    let (bindings, module) = parse_import_from(body)?;
    let module_lower = module.to_lowercase();
    let global = match module_lower.as_str() {
        "pptxgenjs" => "PptxGenJS.default ?? PptxGenJS",
        "exceljs" => "ExcelJS",
        "docx" => "docx",
        "pdf-lib" | "pdflib" => "PDFLib",
        _ => return None,
    };
    if bindings.starts_with('{') {
        return Some(format!("const {bindings} = {global};"));
    }
    if let Some(alias) = bindings.strip_prefix("* as ") {
        let alias = alias.trim();
        if !alias.is_empty() {
            return Some(format!("const {alias} = {global};"));
        }
        return Some(String::new());
    }
    let default_name = global.split('.').next().unwrap_or(global);
    if bindings == default_name {
        Some(String::new())
    } else {
        Some(format!("const {bindings} = {global};"))
    }
}

fn rewrite_require_line(trimmed: &str) -> Option<String> {
    let body = trim_optional_semicolon(trimmed);
    let (kind, rest) = body
        .strip_prefix("const ")
        .map(|r| ("const", r))
        .or_else(|| body.strip_prefix("let ").map(|r| ("let", r)))
        .or_else(|| body.strip_prefix("var ").map(|r| ("var", r)))?;
    let (name, module) = rest.split_once('=')?;
    let name = name.trim();
    let module = module.trim();
    if !module.starts_with("require(") || !module.ends_with(')') {
        return None;
    }
    let id = module
        .strip_prefix("require(")?
        .strip_suffix(')')?
        .trim()
        .trim_matches('"')
        .trim_matches('\'');
    let global = match id.to_lowercase().as_str() {
        "fs" => "fs",
        "path" => "path",
        "exceljs" => "ExcelJS",
        "pptxgenjs" => "PptxGenJS.default ?? PptxGenJS",
        "docx" => "docx",
        "pdf-lib" | "pdflib" => "PDFLib",
        _ => return None,
    };
    if name == global.split('.').next().unwrap_or(global) {
        Some(String::new())
    } else {
        Some(format!("{kind} {name} = {global};"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_trailing_main_call() {
        let code = "async function main() { return 1; }\nmain();\n";
        assert!(!normalize_script(code).contains("main();"));
    }

    #[test]
    fn removes_redundant_exceljs_require() {
        let code = "const ExcelJS = require('exceljs');\nasync function main() {}";
        assert!(!normalize_script(code).contains("require"));
    }

    #[test]
    fn rewrites_aliased_require() {
        let code = "const X = require('exceljs');\nasync function main() { new X.Workbook(); }";
        let n = normalize_script(code);
        assert!(n.contains("const X = ExcelJS"));
        assert!(!n.contains("require"));
    }

    #[test]
    fn wraps_script_without_main() {
        let code = "const x = 1;\nawait Promise.resolve(x);\nreturn x;";
        let n = normalize_script(code);
        assert!(n.contains("async function main()"));
    }

    #[test]
    fn strips_pptxgenjs_require() {
        let code = "const PptxGenJS = require('pptxgenjs');\nconst p = new PptxGenJS();";
        assert!(!normalize_script(code).contains("require"));
    }

    #[test]
    fn strips_fs_require() {
        let code = "const fs = require('fs');\nasync function main() { fs.readFileSync('a.xml','utf-8'); }";
        assert!(!normalize_script(code).contains("require"));
    }

    #[test]
    fn rewrites_import_pptxgenjs_default() {
        let code = "import PptxGenJS from 'pptxgenjs';\nasync function main() { new PptxGenJS(); }";
        let n = normalize_script(code);
        assert!(!n.contains("import "));
        assert!(n.contains("new PptxGenJS()"));
    }

    #[test]
    fn rewrites_import_without_trailing_semicolon() {
        let code = "import PptxGenJS from 'pptxgenjs'\nasync function main() { new PptxGenJS(); }";
        let n = normalize_script(code);
        assert!(!n.contains("import "));
        assert!(n.contains("new PptxGenJS()"));
    }

    #[test]
    fn rewrites_require_without_trailing_semicolon() {
        let code = "const fs = require('fs')\nasync function main() { fs.readFileSync('a.xml', 'utf-8'); }";
        let n = normalize_script(code);
        assert!(!n.contains("require"));
    }

    #[test]
    fn rewrites_import_docx_destructure() {
        let code = "import { Document, Packer } from 'docx';\nasync function main() {}";
        let n = normalize_script(code);
        assert!(n.contains("const { Document, Packer } = docx"));
        assert!(!n.contains("import "));
    }

    #[test]
    fn rewrites_import_exceljs_default() {
        let code =
            "import ExcelJS from 'exceljs';\nasync function main() { new ExcelJS.Workbook(); }";
        assert!(!normalize_script(code).contains("import "));
    }

    #[test]
    fn rewrites_import_pdf_lib_destructure() {
        let code = "import { PDFDocument } from 'pdf-lib';\nasync function main() {}";
        let n = normalize_script(code);
        assert!(n.contains("const { PDFDocument } = PDFLib"));
    }

    #[test]
    fn rewrites_import_namespace_alias() {
        let code = "import * as X from 'pptxgenjs';\nasync function main() { new X(); }";
        let n = normalize_script(code);
        assert!(!n.contains("import "));
        assert!(n.contains("const X = PptxGenJS.default ?? PptxGenJS"));
        assert!(n.contains("new X()"));
    }

    #[test]
    fn unknown_import_left_unchanged() {
        let code = "import foo from 'unknown-pkg';\nasync function main() {}";
        assert!(normalize_script(code).contains("import foo"));
    }
}
