//! Minimal libxml2 + bundled OOXML XSD probe (Windows/macOS/Linux CI).
use libxml::parser::Parser;
use libxml::schemas::SchemaParserContext;
use libxml::schemas::SchemaValidationContext;
use std::path::PathBuf;
use std::process::ExitCode;

fn schemas_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../src-tauri/assets/schemas")
}

fn validate_with_xsd(xsd_path: &PathBuf, xml: &str) -> Result<(), String> {
    let parser = Parser::default();
    let doc = parser
        .parse_string(xml)
        .map_err(|e| format!("xml parse: {e}"))?;
    let mut xsdparser = SchemaParserContext::from_file(xsd_path.to_str().ok_or("bad xsd path")?);
    let mut xsd = SchemaValidationContext::from_parser(&mut xsdparser)
        .map_err(|errs| format!("schema parse ({:?}): {errs:?}", xsd_path))?;
    xsd.validate_document(&doc)
        .map_err(|errs| format!("validation ({:?}): {errs:?}", xsd_path))
}

fn main() -> ExitCode {
    let root = schemas_root();
    if !root.exists() {
        eprintln!("schemas root missing: {}", root.display());
        return ExitCode::FAILURE;
    }

    let wml = root.join("ISO-IEC29500-4_2016/wml.xsd");
    let opc = root.join("ecma/fouth-edition/opc-contentTypes.xsd");

    let valid_doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body><w:p><w:r><w:t>probe</w:t></w:r></w:p></w:body>
</w:document>"#;

    let invalid_doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body><w:tbl><w:tr><w:t>bad nesting</w:t></w:tr></w:tbl></w:body>
</w:document>"#;

    let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#;

    println!("libxml probe: schemas at {}", root.display());

    match validate_with_xsd(&wml, valid_doc) {
        Ok(()) => println!("OK  wml.xsd accepts minimal w:document"),
        Err(e) => {
            eprintln!("FAIL wml valid: {e}");
            return ExitCode::FAILURE;
        }
    }

    match validate_with_xsd(&wml, invalid_doc) {
        Ok(()) => {
            eprintln!("FAIL wml invalid: expected XSD rejection");
            return ExitCode::FAILURE;
        }
        Err(e) => println!("OK  wml.xsd rejects bad nesting: {}", truncate(&e, 120)),
    }

    match validate_with_xsd(&opc, content_types) {
        Ok(()) => println!("OK  opc-contentTypes.xsd accepts [Content_Types].xml"),
        Err(e) => {
            eprintln!("FAIL content types: {e}");
            return ExitCode::FAILURE;
        }
    }

    println!("libxml probe: all checks passed");
    ExitCode::SUCCESS
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}
