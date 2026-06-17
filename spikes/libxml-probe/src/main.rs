//! libxml2 static-link probe: compile bundled OOXML XSD set + smoke validation.
use libxml::parser::Parser;
use libxml::schemas::SchemaParserContext;
use libxml::schemas::SchemaValidationContext;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use walkdir::WalkDir;

fn schemas_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../src-tauri/assets/schemas")
}

fn validate_with_xsd(xsd_path: &Path, xml: &str) -> Result<(), String> {
    let parser = Parser::default();
    let doc = parser
        .parse_string(xml)
        .map_err(|e| format!("xml parse: {e}"))?;
    let mut xsdparser = SchemaParserContext::from_file(
        xsd_path
            .to_str()
            .ok_or_else(|| format!("bad xsd path: {}", xsd_path.display()))?,
    );
    let mut xsd = SchemaValidationContext::from_parser(&mut xsdparser)
        .map_err(|errs| format!("schema parse ({}): {errs:?}", xsd_path.display()))?;
    xsd.validate_document(&doc)
        .map_err(|errs| format!("validation ({}): {errs:?}", xsd_path.display()))
}

fn compile_schema(xsd_path: &Path) -> Result<(), String> {
    let mut spc = SchemaParserContext::from_file(
        xsd_path
            .to_str()
            .ok_or_else(|| format!("bad xsd path: {}", xsd_path.display()))?,
    );
    SchemaValidationContext::from_parser(&mut spc)
        .map(|_| ())
        .map_err(|errs| {
            let msg = errs
                .first()
                .and_then(|e| e.message.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("unknown schema error");
            format!("{}: {msg}", xsd_path.display())
        })
}

fn rel_schema(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn compile_all_schemas(root: &Path) -> Result<(u32, u32, Vec<String>), Vec<String>> {
    let mut paths: Vec<PathBuf> = WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("xsd"))
        .map(|e| e.path().to_path_buf())
        .collect();
    paths.sort();

    let total = paths.len() as u32;
    let mut failures = Vec::new();
    let mut skipped = Vec::new();
    for path in &paths {
        let rel = rel_schema(root, path);
        if is_optional_standalone_schema(&rel) {
            skipped.push(rel);
            continue;
        }
        if let Err(e) = compile_schema(path) {
            failures.push(format!("{rel} — {e}"));
        }
    }

    if failures.is_empty() {
        let compiled = total - skipped.len() as u32;
        Ok((compiled, total, skipped))
    } else {
        Err(failures)
    }
}

/// Schemas not compiled standalone in upstream doc_skills either:
/// - coreProperties pulls Dublin Core XSD from the network
/// - two Microsoft extension stubs fail type resolution unless wml root is loaded first
fn is_optional_standalone_schema(rel: &str) -> bool {
    matches!(
        rel,
        "ecma/fouth-edition/opc-coreProperties.xsd"
            | "microsoft/wml-sdtdatahash-2020.xsd"
            | "microsoft/wml-symex-2015.xsd"
    )
}

fn smoke_validate_documents(root: &Path) -> Result<(), String> {
    let cases: [(&str, &str); 5] = [
        (
            "ISO-IEC29500-4_2016/wml.xsd",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body><w:p><w:r><w:t>probe</w:t></w:r></w:p></w:body>
</w:document>"#,
        ),
        (
            "ISO-IEC29500-4_2016/pml.xsd",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
  xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:sldMasterIdLst/><p:sldIdLst/>
  <p:sldSz cx="9144000" cy="6858000"/>
  <p:notesSz cx="6858000" cy="9144000"/>
</p:presentation>"#,
        ),
        (
            "ISO-IEC29500-4_2016/sml.xsd",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheets><sheet name="Sheet1" sheetId="1" r:id="rId1"
    xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"/></sheets>
</workbook>"#,
        ),
        (
            "ecma/fouth-edition/opc-contentTypes.xsd",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#,
        ),
        (
            "ecma/fouth-edition/opc-relationships.xsd",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#,
        ),
    ];

    for (rel, xml) in cases {
        let xsd = root.join(rel);
        validate_with_xsd(&xsd, xml)?;
    }

    // wml must reject structurally invalid nesting
    let wml = root.join("ISO-IEC29500-4_2016/wml.xsd");
    let invalid = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body><w:tbl><w:tr><w:t>bad</w:t></w:tr></w:tbl></w:body>
</w:document>"#;
    match validate_with_xsd(&wml, invalid) {
        Ok(()) => return Err("wml.xsd should reject invalid nesting".into()),
        Err(_) => {}
    }

    Ok(())
}

fn main() -> ExitCode {
    let root = schemas_root();
    if !root.exists() {
        eprintln!("schemas root missing: {}", root.display());
        return ExitCode::FAILURE;
    }

    println!("libxml probe (static={})", static_link_enabled());
    println!("schemas: {}", root.display());

    match compile_all_schemas(&root) {
        Ok((compiled, total, skipped)) => {
            println!("OK  compiled {compiled}/{total} bundled .xsd schemas");
            for s in skipped {
                println!("    skip optional standalone: {s}");
            }
        }
        Err(failures) => {
            eprintln!("FAIL schema compile ({} required failures):", failures.len());
            for f in &failures {
                eprintln!("  - {f}");
            }
            return ExitCode::FAILURE;
        }
    }

    match smoke_validate_documents(&root) {
        Ok(()) => println!("OK  smoke validation (wml/pml/sml/opc + wml rejection)"),
        Err(e) => {
            eprintln!("FAIL smoke validation: {e}");
            return ExitCode::FAILURE;
        }
    }

    println!("libxml probe: all checks passed");
    ExitCode::SUCCESS
}

fn static_link_enabled() -> bool {
    std::env::var_os("LIBXML2_STATIC").is_some()
        || std::env::var("VCPKG_DEFAULT_TRIPLET")
            .map(|t| t.contains("static"))
            .unwrap_or(false)
}
