//! Built-in Typst templates mounted at `/doc-agent/typst/**`.

pub const VPATH_PREFIX: &str = "/doc-agent/typst";

#[derive(Debug, Clone, Copy)]
pub struct TemplateMeta {
    pub id: &'static str,
    pub category: &'static str,
    pub lang: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub rel_path: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct TemplateFile {
    rel_path: &'static str,
    source: &'static str,
}

const VPATH_COMMON_FONTS: &str = "/doc-agent/typst/common/fonts.typ";
const VPATH_COMMON_FONTS_STACK: &str = "/doc-agent/typst/common/fonts-stack.typ";
const VPATH_COMMON_TOKENS: &str = "/doc-agent/typst/common/tokens.typ";
const VPATH_COMMON_PAGE: &str = "/doc-agent/typst/common/page.typ";
const VPATH_COMMON_EXAM: &str = "/doc-agent/typst/common/exam.typ";
const VPATH_COMMON_LECTURE: &str = "/doc-agent/typst/common/lecture.typ";
const VPATH_REPORT_ZH: &str = "/doc-agent/typst/report/report-zh.typ";
const VPATH_REPORT_EN: &str = "/doc-agent/typst/report/report-en.typ";
const VPATH_EXAM_ZH: &str = "/doc-agent/typst/exam/exam-zh.typ";
const VPATH_EXAM_EN: &str = "/doc-agent/typst/exam/exam-en.typ";
const VPATH_PAPER_ZH: &str = "/doc-agent/typst/paper/paper-zh.typ";
const VPATH_PAPER_EN: &str = "/doc-agent/typst/paper/paper-en.typ";
const VPATH_LECTURE_ZH: &str = "/doc-agent/typst/lecture/lecture-zh.typ";
const VPATH_LECTURE_EN: &str = "/doc-agent/typst/lecture/lecture-en.typ";

#[cfg(target_os = "macos")]
const FONTS_STACK_SOURCE: &str =
    include_str!("../../../assets/typst-templates/common/fonts-stack-macos.typ");
#[cfg(target_os = "windows")]
const FONTS_STACK_SOURCE: &str =
    include_str!("../../../assets/typst-templates/common/fonts-stack-windows.typ");
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const FONTS_STACK_SOURCE: &str =
    include_str!("../../../assets/typst-templates/common/fonts-stack-fallback.typ");

const FILES: &[TemplateFile] = &[
    TemplateFile {
        rel_path: "common/tokens.typ",
        source: include_str!("../../../assets/typst-templates/common/tokens.typ"),
    },
    TemplateFile {
        rel_path: "common/fonts.typ",
        source: include_str!("../../../assets/typst-templates/common/fonts.typ"),
    },
    TemplateFile {
        rel_path: "common/page.typ",
        source: include_str!("../../../assets/typst-templates/common/page.typ"),
    },
    TemplateFile {
        rel_path: "common/exam.typ",
        source: include_str!("../../../assets/typst-templates/common/exam.typ"),
    },
    TemplateFile {
        rel_path: "common/lecture.typ",
        source: include_str!("../../../assets/typst-templates/common/lecture.typ"),
    },
    TemplateFile {
        rel_path: "report/report-zh.typ",
        source: include_str!("../../../assets/typst-templates/report/report-zh.typ"),
    },
    TemplateFile {
        rel_path: "report/report-en.typ",
        source: include_str!("../../../assets/typst-templates/report/report-en.typ"),
    },
    TemplateFile {
        rel_path: "exam/exam-zh.typ",
        source: include_str!("../../../assets/typst-templates/exam/exam-zh.typ"),
    },
    TemplateFile {
        rel_path: "exam/exam-en.typ",
        source: include_str!("../../../assets/typst-templates/exam/exam-en.typ"),
    },
    TemplateFile {
        rel_path: "paper/paper-zh.typ",
        source: include_str!("../../../assets/typst-templates/paper/paper-zh.typ"),
    },
    TemplateFile {
        rel_path: "paper/paper-en.typ",
        source: include_str!("../../../assets/typst-templates/paper/paper-en.typ"),
    },
    TemplateFile {
        rel_path: "lecture/lecture-zh.typ",
        source: include_str!("../../../assets/typst-templates/lecture/lecture-zh.typ"),
    },
    TemplateFile {
        rel_path: "lecture/lecture-en.typ",
        source: include_str!("../../../assets/typst-templates/lecture/lecture-en.typ"),
    },
    TemplateFile {
        rel_path: "syntax/typst-guide.md",
        source: include_str!("../../../assets/typst-templates/syntax/typst-guide.md"),
    },
];

pub const LISTABLE: &[TemplateMeta] = &[
    TemplateMeta {
        id: "syntax/typst-guide",
        category: "syntax",
        lang: "zh",
        title: "Typst 语法手册",
        description: "通用 Typst 0.13 语法参考；调用 Typst 能力前必读",
        rel_path: "syntax/typst-guide.md",
    },
    TemplateMeta {
        id: "report/report-zh",
        category: "report",
        lang: "zh",
        title: "中文技术报告",
        description: "含摘要、目录、对比表与结论章节",
        rel_path: "report/report-zh.typ",
    },
    TemplateMeta {
        id: "report/report-en",
        category: "report",
        lang: "en",
        title: "Technical Report (EN)",
        description: "Executive summary, TOC, comparison table, conclusion",
        rel_path: "report/report-en.typ",
    },
    TemplateMeta {
        id: "exam/exam-zh",
        category: "exam",
        lang: "zh",
        title: "中文试卷",
        description: "高等数学风格：填空横线、选择题、计算证明；见 common/exam.typ",
        rel_path: "exam/exam-zh.typ",
    },
    TemplateMeta {
        id: "exam/exam-en",
        category: "exam",
        lang: "en",
        title: "Exam Paper (EN)",
        description: "Calculus-style fill-in, multiple choice, problems",
        rel_path: "exam/exam-en.typ",
    },
    TemplateMeta {
        id: "paper/paper-zh",
        category: "paper",
        lang: "zh",
        title: "中文学术论文",
        description: "摘要、关键词、章节编号、公式与参考文献",
        rel_path: "paper/paper-zh.typ",
    },
    TemplateMeta {
        id: "paper/paper-en",
        category: "paper",
        lang: "en",
        title: "Academic Paper (EN)",
        description: "Abstract, keywords, numbered sections, references",
        rel_path: "paper/paper-en.typ",
    },
    TemplateMeta {
        id: "lecture/lecture-zh",
        category: "lecture",
        lang: "zh",
        title: "中文讲义",
        description: "定义/例题区块、表格与课堂练习",
        rel_path: "lecture/lecture-zh.typ",
    },
    TemplateMeta {
        id: "lecture/lecture-en",
        category: "lecture",
        lang: "en",
        title: "Lecture Notes (EN)",
        description: "Definition/example blocks, tables, exercises",
        rel_path: "lecture/lecture-en.typ",
    },
];

pub fn vpath(rel_path: &str) -> String {
    format!("{VPATH_PREFIX}/{rel_path}")
}

pub fn scene_template_ids() -> &'static [&'static str] {
    &[
        "report/report-zh",
        "report/report-en",
        "exam/exam-zh",
        "exam/exam-en",
        "paper/paper-zh",
        "paper/paper-en",
        "lecture/lecture-zh",
        "lecture/lecture-en",
    ]
}

pub fn typst_guide_source() -> &'static str {
    source_for_rel("syntax/typst-guide.md")
}

pub fn static_sources() -> Vec<(&'static str, &'static str)> {
    vec![
        (VPATH_COMMON_FONTS_STACK, FONTS_STACK_SOURCE),
        (VPATH_COMMON_TOKENS, source_for_rel("common/tokens.typ")),
        (VPATH_COMMON_FONTS, source_for_rel("common/fonts.typ")),
        (VPATH_COMMON_PAGE, source_for_rel("common/page.typ")),
        (VPATH_COMMON_EXAM, source_for_rel("common/exam.typ")),
        (VPATH_COMMON_LECTURE, source_for_rel("common/lecture.typ")),
        (VPATH_REPORT_ZH, source_for_rel("report/report-zh.typ")),
        (VPATH_REPORT_EN, source_for_rel("report/report-en.typ")),
        (VPATH_EXAM_ZH, source_for_rel("exam/exam-zh.typ")),
        (VPATH_EXAM_EN, source_for_rel("exam/exam-en.typ")),
        (VPATH_PAPER_ZH, source_for_rel("paper/paper-zh.typ")),
        (VPATH_PAPER_EN, source_for_rel("paper/paper-en.typ")),
        (VPATH_LECTURE_ZH, source_for_rel("lecture/lecture-zh.typ")),
        (VPATH_LECTURE_EN, source_for_rel("lecture/lecture-en.typ")),
    ]
}

pub fn find_by_id(id: &str) -> Option<&'static TemplateMeta> {
    LISTABLE.iter().find(|m| m.id == id)
}

pub fn find_source(id: &str) -> Option<&'static str> {
    if let Some(meta) = find_by_id(id) {
        return FILES
            .iter()
            .find(|f| f.rel_path == meta.rel_path)
            .map(|f| f.source);
    }
    FILES
        .iter()
        .find(|f| f.rel_path == id || vpath(f.rel_path) == id)
        .map(|f| f.source)
}

fn source_for_rel(rel_path: &str) -> &'static str {
    FILES
        .iter()
        .find(|f| f.rel_path == rel_path)
        .unwrap_or_else(|| panic!("missing bundled typst file: {rel_path}"))
        .source
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_sources_include_tokens_and_lecture() {
        let paths: Vec<_> = static_sources().into_iter().map(|(p, _)| p).collect();
        assert!(paths.contains(&VPATH_COMMON_TOKENS));
        assert!(paths.contains(&VPATH_COMMON_LECTURE));
        assert!(paths.contains(&VPATH_REPORT_ZH));
    }
}
