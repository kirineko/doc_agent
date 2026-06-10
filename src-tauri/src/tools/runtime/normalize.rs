/// 执行前规范化模型生成的 skill_run 脚本，兼容常见 Node 写法。
pub fn normalize_script(code: &str) -> String {
    let mut out = String::new();
    for line in code.lines() {
        if let Some(replacement) = rewrite_require_line(line.trim()) {
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

fn rewrite_require_line(trimmed: &str) -> Option<String> {
    let body = trimmed.strip_suffix(';')?.trim();
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
}
