#[derive(Clone, Copy)]
pub struct SkillDoc {
    pub name: &'static str,
    pub content: &'static str,
}

#[derive(Clone, Copy)]
pub struct Skill {
    pub name: &'static str,
    pub description: &'static str,
    pub docs: &'static [SkillDoc],
}

static DOCX_DOCS: &[SkillDoc] = &[
    SkillDoc {
        name: "SKILL.md",
        content: include_str!("../../assets/skills/docx/SKILL.md"),
    },
    SkillDoc {
        name: "editing.md",
        content: include_str!("../../assets/skills/docx/editing.md"),
    },
    SkillDoc {
        name: "math.md",
        content: include_str!("../../assets/skills/docx/math.md"),
    },
];

static PDF_DOCS: &[SkillDoc] = &[
    SkillDoc {
        name: "SKILL.md",
        content: include_str!("../../assets/skills/pdf/SKILL.md"),
    },
    SkillDoc {
        name: "reference.md",
        content: include_str!("../../assets/skills/pdf/reference.md"),
    },
    SkillDoc {
        name: "forms.md",
        content: include_str!("../../assets/skills/pdf/forms.md"),
    },
];

static PPTX_DOCS: &[SkillDoc] = &[
    SkillDoc {
        name: "SKILL.md",
        content: include_str!("../../assets/skills/pptx/SKILL.md"),
    },
    SkillDoc {
        name: "pptxgenjs.md",
        content: include_str!("../../assets/skills/pptx/pptxgenjs.md"),
    },
    SkillDoc {
        name: "editing.md",
        content: include_str!("../../assets/skills/pptx/editing.md"),
    },
];

static XLSX_DOCS: &[SkillDoc] = &[SkillDoc {
    name: "SKILL.md",
    content: include_str!("../../assets/skills/xlsx/SKILL.md"),
}];

static HTML_REPORT_DOCS: &[SkillDoc] = &[SkillDoc {
    name: "SKILL.md",
    content: include_str!("../../assets/skills/html-report/SKILL.md"),
}];

static CLARIFY_DOCS: &[SkillDoc] = &[SkillDoc {
    name: "SKILL.md",
    content: include_str!("../../assets/skills/clarify/SKILL.md"),
}];

static RUNTIME_DOCS: &[SkillDoc] = &[SkillDoc {
    name: "SKILL.md",
    content: include_str!("../../assets/skills/runtime/SKILL.md"),
}];

static PROFILE_DOCS: &[SkillDoc] = &[SkillDoc {
    name: "SKILL.md",
    content: include_str!("../../assets/skills/profile/SKILL.md"),
}];

pub static SKILLS: &[Skill] = &[
    Skill {
        name: "docx",
        description: "Word 文档创建、读取、编辑、修订、批注与表格",
        docs: DOCX_DOCS,
    },
    Skill {
        name: "pdf",
        description: "PDF 智能读取（pdf_read）、页面操作、表格提取与表单处理",
        docs: PDF_DOCS,
    },
    Skill {
        name: "pptx",
        description: "PPT 创建、读取、模板编辑与幻灯片操作",
        docs: PPTX_DOCS,
    },
    Skill {
        name: "xlsx",
        description: "Excel 分析、公式模型、样式化表格生成与校验",
        docs: XLSX_DOCS,
    },
    Skill {
        name: "html-report",
        description: "静态 HTML 网页报告（表格、文字、CSS）生成规范与落盘约定",
        docs: HTML_REPORT_DOCS,
    },
    Skill {
        name: "clarify",
        description: "文档创作前的需求澄清流程，帮助明确内容、结构与排版风格",
        docs: CLARIFY_DOCS,
    },
    Skill {
        name: "runtime",
        description: "skill_run 嵌入式 JS 运行时 API、polyfill 与限制（编写脚本前必读）",
        docs: RUNTIME_DOCS,
    },
    Skill {
        name: "profile",
        description: "项目 AGENTS.md 配置：/init 澄清流程与写入规范",
        docs: PROFILE_DOCS,
    },
];

pub fn index_markdown() -> String {
    let mut out = String::from("可用 Document Skills（处理复杂文档前先 skill_read 获取全文）：\n");
    for skill in SKILLS {
        out.push_str(&format!("- **{}**: {}\n", skill.name, skill.description));
    }
    out
}

/// 解析 skill/doc 参数，兼容模型把 `pptxgenjs.md` 误填到 skill 字段的情况。
pub fn resolve_skill_doc(skill: &str, doc: Option<&str>) -> Result<(String, String), String> {
    if let Some(doc_name) = doc {
        return Ok((skill.to_string(), doc_name.to_string()));
    }
    if skill.ends_with(".md") {
        let mut owners: Vec<&str> = Vec::new();
        for entry in SKILLS {
            if entry.docs.iter().any(|d| d.name == skill) {
                owners.push(entry.name);
            }
        }
        return match owners.len() {
            0 => Err(format!(
                "未找到文档 '{skill}'。请使用 {{\"skill\":\"docx\",\"doc\":\"{skill}\"}} 形式"
            )),
            1 => Ok((owners[0].to_string(), skill.to_string())),
            _ => Err(format!(
                "文档 '{skill}' 同时属于 skill: {}。请显式指定 skill 参数",
                owners.join(", ")
            )),
        };
    }
    match skill {
        "pptxgenjs" => Ok(("pptx".into(), "pptxgenjs.md".into())),
        "editing" => Ok(("docx".into(), "editing.md".into())),
        "math" => Ok(("docx".into(), "math.md".into())),
        "reference" => Ok(("pdf".into(), "reference.md".into())),
        "forms" => Ok(("pdf".into(), "forms.md".into())),
        _ => Ok((skill.to_string(), "SKILL.md".into())),
    }
}

pub fn read(skill: &str, doc: Option<&str>) -> Result<&'static str, String> {
    let (skill_name, doc_name) = resolve_skill_doc(skill, doc)?;
    let skill_name = skill_name.as_str();
    let doc_name = doc_name.as_str();
    let entry = SKILLS
        .iter()
        .find(|s| s.name == skill_name)
        .ok_or_else(|| format_available_skills(skill_name))?;
    entry
        .docs
        .iter()
        .find(|d| d.name == doc_name)
        .map(|d| d.content)
        .ok_or_else(|| {
            let names: Vec<_> = entry.docs.iter().map(|d| d.name).collect();
            format!(
                "skill '{skill_name}' 无文档 '{doc_name}'，可用: {}",
                names.join(", ")
            )
        })
}

pub fn available_names() -> Vec<&'static str> {
    SKILLS.iter().map(|s| s.name).collect()
}

fn format_available_skills(unknown: &str) -> String {
    format!(
        "未知 skill '{unknown}'，可用: {}",
        available_names().join(", ")
    )
}
