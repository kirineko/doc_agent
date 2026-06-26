use super::rules::{self, opc, pml, sml, wml};

const MIN_CT: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#;

const MIN_DOC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body><w:p><w:r><w:t>ok</w:t></w:r></w:p></w:body>
</w:document>"#;

const MIN_PRES: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
  xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:sldMasterIdLst/><p:sldIdLst/>
  <p:sldSz cx="9144000" cy="6858000"/>
  <p:notesSz cx="6858000" cy="9144000"/>
</p:presentation>"#;

const MIN_WB: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"
  xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets><sheet name="Sheet1" sheetId="1" r:id="rId1"/></sheets>
</workbook>"#;

const MIN_WS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData><row r="1"><c r="A1"><v>1</v></c></row></sheetData>
</worksheet>"#;

#[test]
fn wml_valid_min_document_passes() {
    assert!(wml::validate_wml_document(MIN_DOC).is_empty());
}

#[test]
fn wml_invalid_tr_direct_t_fails() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body><w:tbl><w:tr><w:t>x</w:t></w:tr></w:tbl></w:body>
</w:document>"#;
    let v = wml::validate_wml_document(xml);
    assert!(v.iter().any(|e| e.rule_id == "wml.tr.01"));
}

#[test]
fn pml_missing_notes_sz_fails() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:sldIdLst/>
</p:presentation>"#;
    let v = pml::validate_presentation(xml);
    assert!(v.iter().any(|e| e.rule_id == "pml.pres.03"));
}

#[test]
fn pml_valid_min_presentation_passes() {
    assert!(pml::validate_presentation(MIN_PRES).is_empty());
}

#[test]
fn sml_missing_sheet_data_fails() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <dimension ref="A1"/>
</worksheet>"#;
    let v = sml::validate_worksheet(xml);
    assert!(v.iter().any(|e| e.rule_id == "sml.ws.02"));
}

#[test]
fn sml_valid_min_workbook_and_sheet_pass() {
    assert!(sml::validate_workbook(MIN_WB).is_empty());
    assert!(sml::validate_worksheet(MIN_WS).is_empty());
}

#[test]
fn opc_rels_duplicate_id_fails() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://t" Target="a.xml"/>
  <Relationship Id="rId1" Type="http://t" Target="b.xml"/>
</Relationships>"#;
    let v = opc::validate_relationships(xml);
    assert!(v.iter().any(|e| e.rule_id == "opc.rels.03"));
}

#[test]
fn opc_content_types_valid_passes() {
    assert!(opc::validate_content_types(MIN_CT).is_empty());
}

#[test]
fn opc_content_types_prefixed_root_passes() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ct:Types xmlns:ct="http://schemas.openxmlformats.org/package/2006/content-types">
  <ct:Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <ct:Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</ct:Types>"#;
    assert!(opc::validate_content_types(xml).is_empty());
}

#[test]
fn validate_dir_docx_minimal_unpack_layout() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path();
    std::fs::write(base.join("[Content_Types].xml"), MIN_CT).unwrap();
    std::fs::create_dir_all(base.join("word")).unwrap();
    std::fs::write(base.join("word/document.xml"), MIN_DOC).unwrap();
    std::fs::create_dir_all(base.join("word/_rels")).unwrap();
    std::fs::write(
        base.join("word/_rels/document.xml.rels"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"/>"#,
    )
    .unwrap();
    let index = rules::parse_content_types(MIN_CT).unwrap();
    let missing = rules::validate_package_parts(base, &index);
    assert!(
        missing.is_empty(),
        "unexpected part registration errors: {:?}",
        missing
    );
    super::validate_dir(base, None).expect("minimal docx layout should validate");
}

#[test]
fn tbl_missing_grid_fails() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body><w:tbl><w:tblPr/><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl></w:body>
</w:document>"#;
    let v = wml::validate_wml_document(xml);
    assert!(v.iter().any(|e| e.rule_id == "wml.tbl.01"));
}

#[test]
fn tbl_sdt_wrapped_tr_still_requires_grid() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body><w:tbl><w:tblPr/><w:sdt><w:tr><w:tc><w:p/></w:tc></w:tr></w:sdt></w:tbl></w:body>
</w:document>"#;
    let v = wml::validate_wml_document(xml);
    assert!(v.iter().any(|e| e.rule_id == "wml.tbl.01"));
}

#[test]
fn tc_with_ins_wrapped_paragraph_passes() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body><w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:ins><w:p/></w:ins></w:tc></w:tr></w:tbl></w:body>
</w:document>"#;
    let v = wml::validate_wml_document(xml);
    assert!(!v.iter().any(|e| e.rule_id == "wml.tc.01"));
}

#[test]
fn nested_tbl_outer_tr_after_inner_still_checked() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body><w:tbl>
    <w:tblPr/><w:tblGrid/>
    <w:tr><w:tc>
      <w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl>
    </w:tc></w:tr>
    <w:tr><w:tc/></w:tr>
  </w:tbl></w:body>
</w:document>"#;
    let v = wml::validate_wml_document(xml);
    assert!(v.iter().any(|e| e.rule_id == "wml.tc.01"));
}

#[test]
fn nested_tbl_outer_missing_grid_after_inner_closes() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body><w:tbl>
    <w:tr><w:tc>
      <w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl>
    </w:tc></w:tr>
    <w:tr><w:tc><w:p/></w:tc></w:tr>
  </w:tbl></w:body>
</w:document>"#;
    let v = wml::validate_wml_document(xml);
    assert!(v.iter().any(|e| e.rule_id == "wml.tbl.01"));
}

#[test]
fn nested_tbl_empty_inner_tc_fails() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body><w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc>
    <w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc/></w:tr></w:tbl>
  </w:tc></w:tr></w:tbl></w:body>
</w:document>"#;
    let v = wml::validate_wml_document(xml);
    assert_eq!(v.iter().filter(|e| e.rule_id == "wml.tc.01").count(), 1);
}

#[test]
fn wml_math_and_revision_content_do_not_false_fail() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
  xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
  <w:body>
    <m:oMath/>
    <w:moveFrom/>
    <w:tbl>
      <w:tblPr/><w:tblGrid/>
      <w:moveTo/>
      <w:tr><w:tc><m:oMath/></w:tc></w:tr>
    </w:tbl>
  </w:body>
</w:document>"#;
    let v = wml::validate_wml_document(xml);
    assert!(
        !v.iter().any(|e| matches!(
            e.rule_id,
            "wml.body.01" | "wml.tbl.02" | "wml.tr.01" | "wml.tc.01"
        )),
        "unexpected WML false-positive errors: {:?}",
        v
    );
}

#[test]
fn pml_markup_compatibility_wrappers_do_not_false_fail() {
    let pres = r#"<?xml version="1.0" encoding="UTF-8"?>
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
  xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"
  xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <mc:AlternateContent>
    <mc:Choice Requires="p"><p:sldIdLst/></mc:Choice>
    <mc:Fallback><p:sldIdLst/></mc:Fallback>
  </mc:AlternateContent>
  <p:notesSz cx="6858000" cy="9144000"/>
</p:presentation>"#;
    assert!(pml::validate_presentation(pres).is_empty());

    let slide = r#"<?xml version="1.0" encoding="UTF-8"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
  xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006">
  <mc:AlternateContent>
    <mc:Choice Requires="p"><p:cSld><p:spTree/></p:cSld></mc:Choice>
    <mc:Fallback><p:cSld><p:spTree/></p:cSld></mc:Fallback>
  </mc:AlternateContent>
</p:sld>"#;
    assert!(pml::validate_slide(slide).is_empty());
}

#[test]
fn pml_mc_wrapped_notes_size_counts_as_one_effective_child() {
    let pres = r#"<?xml version="1.0" encoding="UTF-8"?>
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
  xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006">
  <mc:AlternateContent>
    <mc:Choice Requires="p"><p:notesSz cx="6858000" cy="9144000"/></mc:Choice>
    <mc:Fallback><p:notesSz cx="6858000" cy="9144000"/></mc:Fallback>
  </mc:AlternateContent>
</p:presentation>"#;
    assert!(pml::validate_presentation(pres).is_empty());
}

#[test]
fn sml_markup_compatibility_wrappers_do_not_false_fail() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"
  xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006">
  <sheetData>
    <mc:AlternateContent>
      <mc:Choice Requires="x"><row r="1"><c r="A1"/></row></mc:Choice>
      <mc:Fallback><row r="1"><c r="A1"/></row></mc:Fallback>
    </mc:AlternateContent>
  </sheetData>
</worksheet>"#;
    assert!(sml::validate_worksheet(xml).is_empty());
}

#[test]
fn sml_mc_wrapped_required_children_count_as_one_effective_child() {
    let workbook = r#"<?xml version="1.0" encoding="UTF-8"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"
  xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
  xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006">
  <mc:AlternateContent>
    <mc:Choice Requires="x"><sheets><sheet name="Sheet1" sheetId="1" r:id="rId1"/></sheets></mc:Choice>
    <mc:Fallback><sheets><sheet name="Sheet1" sheetId="1" r:id="rId1"/></sheets></mc:Fallback>
  </mc:AlternateContent>
</workbook>"#;
    assert!(sml::validate_workbook(workbook).is_empty());

    let worksheet = r#"<?xml version="1.0" encoding="UTF-8"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"
  xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006">
  <mc:AlternateContent>
    <mc:Choice Requires="x"><sheetData><row r="1"><c r="A1"/></row></sheetData></mc:Choice>
    <mc:Fallback><sheetData><row r="1"><c r="A1"/></row></sheetData></mc:Fallback>
  </mc:AlternateContent>
</worksheet>"#;
    assert!(sml::validate_worksheet(worksheet).is_empty());
}

#[test]
fn opc_ct05_violation_reports_actual_part() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("[Content_Types].xml"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#,
    )
    .unwrap();
    std::fs::create_dir_all(dir.path().join("word")).unwrap();
    std::fs::write(dir.path().join("word/header1.xml"), "<x/>").unwrap();
    let index = rules::parse_content_types(
        &std::fs::read_to_string(dir.path().join("[Content_Types].xml")).unwrap(),
    )
    .unwrap();
    let v = rules::validate_package_parts(dir.path(), &index);
    assert!(v.iter().any(|e| e.rule_id == "opc.ct.05"));
    let msg = super::error::violations_to_error("[Content_Types].xml", v);
    assert!(msg.starts_with("word/header1.xml"));
}

fn write_comments_xml(dir: &tempfile::TempDir, ids: &[&str]) {
    std::fs::create_dir_all(dir.path().join("word")).unwrap();
    let entries: String = ids
        .iter()
        .map(|id| format!(r#"<w:comment w:id="{id}"/>"#))
        .collect();
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{entries}</w:comments>"#
    );
    std::fs::write(dir.path().join("word/comments.xml"), xml).unwrap();
}

fn doc_with_refs(ids: &[&str]) -> String {
    let refs: String = ids
        .iter()
        .map(|id| format!(r#"<w:r><w:commentReference w:id="{id}"/></w:r>"#))
        .collect();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p>{refs}</w:p></w:body></w:document>"#
    )
}

#[test]
fn comment_consistency_consistent_passes() {
    let dir = tempfile::tempdir().unwrap();
    write_comments_xml(&dir, &["1", "2"]);
    let v = wml::validate_comment_consistency(dir.path(), &doc_with_refs(&["1", "2"]));
    assert!(v.is_empty(), "expected no violations, got {v:?}");
}

#[test]
fn comment_consistency_comment_without_reference_fails() {
    let dir = tempfile::tempdir().unwrap();
    write_comments_xml(&dir, &["1"]);
    let v = wml::validate_comment_consistency(dir.path(), &doc_with_refs(&[]));
    assert!(
        v.iter()
            .any(|e| e.rule_id == "wml.comment.consistency" && e.message.contains("1")),
        "expected violation for id 1, got {v:?}"
    );
}

#[test]
fn comment_consistency_reference_without_comment_fails() {
    let dir = tempfile::tempdir().unwrap();
    write_comments_xml(&dir, &[]);
    let v = wml::validate_comment_consistency(dir.path(), &doc_with_refs(&["2"]));
    assert!(
        v.iter()
            .any(|e| e.rule_id == "wml.comment.consistency" && e.message.contains("2")),
        "expected violation for id 2, got {v:?}"
    );
}

#[test]
fn comment_consistency_no_comments_part_passes() {
    let dir = tempfile::tempdir().unwrap();
    // No word/comments.xml at all; document has no refs either.
    let v = wml::validate_comment_consistency(dir.path(), &doc_with_refs(&[]));
    assert!(v.is_empty(), "got {v:?}");
}

#[test]
fn comment_consistency_dangling_ref_when_comments_part_missing() {
    let dir = tempfile::tempdir().unwrap();
    // No word/comments.xml, but document.xml references id "3" — that reference
    // cannot resolve and must be reported (regression for the early-return that
    // used to treat a missing comments part as "nothing to check").
    let v = wml::validate_comment_consistency(dir.path(), &doc_with_refs(&["3"]));
    assert!(
        v.iter()
            .any(|e| e.rule_id == "wml.comment.consistency" && e.message.contains("3")),
        "expected violation for dangling id 3, got {v:?}"
    );
}

#[test]
fn comment_consistency_reference_in_header_passes() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("word/_rels")).unwrap();
    write_comments_xml(&dir, &["1"]);
    // The comment is anchored in a header story, not the main body. Its
    // commentReference lives in word/header1.xml, so document.xml has none.
    std::fs::write(
        dir.path().join("word/header1.xml"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r><w:commentReference w:id="1"/></w:r></w:p></w:hdr>"#,
    )
    .unwrap();
    std::fs::write(
        dir.path().join("word/_rels/document.xml.rels"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="header1.xml"/></Relationships>"#,
    )
    .unwrap();
    let doc = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:body><w:p><w:pPr><w:headerReference w:type="default" r:id="rId1"/></w:pPr></w:p></w:body></w:document>"#;
    let v = wml::validate_comment_consistency(dir.path(), doc);
    assert!(
        v.is_empty(),
        "comment referenced in a reachable header must not be flagged; got {v:?}"
    );
}

#[test]
fn comment_consistency_stale_header_rel_without_header_reference_fails() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("word/_rels")).unwrap();
    write_comments_xml(&dir, &["1"]);
    std::fs::write(
        dir.path().join("word/header1.xml"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r><w:commentReference w:id="1"/></w:r></w:p></w:hdr>"#,
    )
    .unwrap();
    // rels points at header1.xml but document.xml never references rId1 — Word
    // does not load that header, so the commentReference there must not count.
    std::fs::write(
        dir.path().join("word/_rels/document.xml.rels"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="header1.xml"/></Relationships>"#,
    )
    .unwrap();
    let v = wml::validate_comment_consistency(dir.path(), &doc_with_refs(&[]));
    assert!(
        v.iter()
            .any(|e| e.rule_id == "wml.comment.consistency" && e.message.contains("1")),
        "stale header rel without headerReference must not satisfy anchor; got {v:?}"
    );
}

#[test]
fn comment_consistency_unreferenced_header_does_not_count() {
    let dir = tempfile::tempdir().unwrap();
    write_comments_xml(&dir, &["1"]);
    // Stale header1.xml on disk with a commentReference, but no document.xml.rels
    // entry — Word never loads it, so id=1 still lacks a reachable anchor.
    std::fs::write(
        dir.path().join("word/header1.xml"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r><w:commentReference w:id="1"/></w:r></w:p></w:hdr>"#,
    )
    .unwrap();
    let v = wml::validate_comment_consistency(dir.path(), &doc_with_refs(&[]));
    assert!(
        v.iter()
            .any(|e| e.rule_id == "wml.comment.consistency" && e.message.contains("1")),
        "unreferenced header must not satisfy anchor check; got {v:?}"
    );
}

#[test]
fn comment_consistency_reply_without_reference_passes() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("word")).unwrap();
    // id=1 is top-level (paraId AAAA1111); id=2 is a reply to it (paraId
    // BBBB2222). Word links the reply via commentsExtended (paraIdParent) and
    // does NOT emit a commentReference for id=2 in document.xml.
    std::fs::write(
        dir.path().join("word/comments.xml"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml">
  <w:comment w:id="1"><w:p w14:paraId="AAAA1111"><w:r><w:t>top</w:t></w:r></w:p></w:comment>
  <w:comment w:id="2"><w:p w14:paraId="BBBB2222"><w:r><w:t>reply</w:t></w:r></w:p></w:comment>
</w:comments>"#,
    )
    .unwrap();
    std::fs::write(
        dir.path().join("word/commentsExtended.xml"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<w15:commentsEx xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml"><w15:commentEx w15:paraId="BBBB2222" w15:paraIdParent="AAAA1111" w15:done="0"/></w15:commentsEx>"#,
    )
    .unwrap();
    // document.xml references only the top-level comment id=1.
    let v = wml::validate_comment_consistency(dir.path(), &doc_with_refs(&["1"]));
    assert!(
        v.is_empty(),
        "reply comment must not require its own commentReference; got {v:?}"
    );
}

#[test]
fn comment_consistency_non_reply_without_reference_still_fails() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("word")).unwrap();
    // commentsExtended exists but neither commentEx has a paraIdParent, so no
    // comment is a reply: a missing reference must still be reported.
    std::fs::write(
        dir.path().join("word/comments.xml"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml">
  <w:comment w:id="1"><w:p w14:paraId="AAAA1111"><w:r><w:t>top</w:t></w:r></w:p></w:comment>
</w:comments>"#,
    )
    .unwrap();
    std::fs::write(
        dir.path().join("word/commentsExtended.xml"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<w15:commentsEx xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml"><w15:commentEx w15:paraId="AAAA1111" w15:done="0"/></w15:commentsEx>"#,
    )
    .unwrap();
    let v = wml::validate_comment_consistency(dir.path(), &doc_with_refs(&[]));
    assert!(
        v.iter()
            .any(|e| e.rule_id == "wml.comment.consistency" && e.message.contains("1")),
        "expected violation for unanchored non-reply id 1, got {v:?}"
    );
}

#[test]
fn comment_consistency_stale_para_id_parent_still_fails() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("word")).unwrap();
    // commentsExtended claims id=2 is a reply (BBBB2222 → DEADBEEF), but DEADBEEF
    // is not any comment's paraId — stale metadata must not exempt the reference check.
    std::fs::write(
        dir.path().join("word/comments.xml"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml">
  <w:comment w:id="1"><w:p w14:paraId="AAAA1111"><w:r><w:t>top</w:t></w:r></w:p></w:comment>
  <w:comment w:id="2"><w:p w14:paraId="BBBB2222"><w:r><w:t>orphan reply</w:t></w:r></w:p></w:comment>
</w:comments>"#,
    )
    .unwrap();
    std::fs::write(
        dir.path().join("word/commentsExtended.xml"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<w15:commentsEx xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml"><w15:commentEx w15:paraId="BBBB2222" w15:paraIdParent="DEADBEEF" w15:done="0"/></w15:commentsEx>"#,
    )
    .unwrap();
    let v = wml::validate_comment_consistency(dir.path(), &doc_with_refs(&["1"]));
    assert!(
        v.iter()
            .any(|e| e.rule_id == "wml.comment.consistency" && e.message.contains("2")),
        "stale paraIdParent must not exempt id 2 from reference check; got {v:?}"
    );
}
