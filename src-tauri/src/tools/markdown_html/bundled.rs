//! Built-in Markdown HTML templates and samples.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Slide,
    Report,
    Resume,
}

impl Profile {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "slide" => Some(Self::Slide),
            "report" => Some(Self::Report),
            "resume" => Some(Self::Resume),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Slide => "slide",
            Self::Report => "report",
            Self::Resume => "resume",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TemplateMeta {
    pub id: &'static str,
    pub profile: Profile,
    pub title: &'static str,
    pub description: &'static str,
    pub css_rel: &'static str,
    pub sample_rel: Option<&'static str>,
    pub is_default: bool,
}

#[derive(Debug, Clone, Copy)]
struct TemplateFile {
    rel_path: &'static str,
    source: &'static str,
}

const BASE_CSS: &str = include_str!("../../../assets/markdown-templates/common/base.css");
const SLIDE_DEFAULT: &str = include_str!("../../../assets/markdown-templates/slide/default.css");
const SLIDE_GAIA: &str = include_str!("../../../assets/markdown-templates/slide/gaia.css");
const SLIDE_UNCOVER: &str = include_str!("../../../assets/markdown-templates/slide/uncover.css");
const SLIDE_CORPORATE: &str =
    include_str!("../../../assets/markdown-templates/slide/corporate.css");
const SLIDE_ACADEMIC: &str = include_str!("../../../assets/markdown-templates/slide/academic.css");
const SLIDE_MINIMAL_DARK: &str =
    include_str!("../../../assets/markdown-templates/slide/minimal-dark.css");

const REPORT_GITHUB_LIGHT: &str =
    include_str!("../../../assets/markdown-templates/report/github-light.css");
const REPORT_GITHUB_DARK: &str =
    include_str!("../../../assets/markdown-templates/report/github-dark.css");
const REPORT_ACADEMIC: &str =
    include_str!("../../../assets/markdown-templates/report/academic.css");
const REPORT_BUSINESS: &str =
    include_str!("../../../assets/markdown-templates/report/business.css");
const REPORT_NARROW: &str = include_str!("../../../assets/markdown-templates/report/narrow.css");
const REPORT_DATA: &str = include_str!("../../../assets/markdown-templates/report/data.css");

const RESUME_CLASSIC: &str = include_str!("../../../assets/markdown-templates/resume/classic.css");
const RESUME_MODERN: &str = include_str!("../../../assets/markdown-templates/resume/modern.css");
const RESUME_TWO_COL: &str = include_str!("../../../assets/markdown-templates/resume/two-col.css");
const RESUME_COMPACT: &str = include_str!("../../../assets/markdown-templates/resume/compact.css");
const RESUME_EVEN: &str = include_str!("../../../assets/markdown-templates/resume/even.css");

const SAMPLE_SLIDE: &str =
    include_str!("../../../assets/markdown-templates/samples/slide-default.md");
const SAMPLE_REPORT: &str =
    include_str!("../../../assets/markdown-templates/samples/report-github-light.md");
const SAMPLE_RESUME: &str =
    include_str!("../../../assets/markdown-templates/samples/resume-classic.md");

pub const LISTABLE: &[TemplateMeta] = &[
    TemplateMeta {
        id: "slide/default",
        profile: Profile::Slide,
        title: "Default",
        description: "Marp 默认浅色主题",
        css_rel: "slide/default.css",
        sample_rel: Some("samples/slide-default.md"),
        is_default: true,
    },
    TemplateMeta {
        id: "slide/gaia",
        profile: Profile::Slide,
        title: "Gaia",
        description: "柔和渐变背景",
        css_rel: "slide/gaia.css",
        sample_rel: None,
        is_default: false,
    },
    TemplateMeta {
        id: "slide/uncover",
        profile: Profile::Slide,
        title: "Uncover",
        description: "深色全屏演示",
        css_rel: "slide/uncover.css",
        sample_rel: None,
        is_default: false,
    },
    TemplateMeta {
        id: "slide/corporate",
        profile: Profile::Slide,
        title: "Corporate",
        description: "商务绿强调色",
        css_rel: "slide/corporate.css",
        sample_rel: None,
        is_default: false,
    },
    TemplateMeta {
        id: "slide/academic",
        profile: Profile::Slide,
        title: "Academic",
        description: "学术 serif 正文",
        css_rel: "slide/academic.css",
        sample_rel: None,
        is_default: false,
    },
    TemplateMeta {
        id: "slide/minimal-dark",
        profile: Profile::Slide,
        title: "Minimal Dark",
        description: "极简深色代码风格",
        css_rel: "slide/minimal-dark.css",
        sample_rel: None,
        is_default: false,
    },
    TemplateMeta {
        id: "report/github-light",
        profile: Profile::Report,
        title: "GitHub Light",
        description: "GitHub 风格浅色报告",
        css_rel: "report/github-light.css",
        sample_rel: Some("samples/report-github-light.md"),
        is_default: true,
    },
    TemplateMeta {
        id: "report/github-dark",
        profile: Profile::Report,
        title: "GitHub Dark",
        description: "GitHub 风格深色报告",
        css_rel: "report/github-dark.css",
        sample_rel: None,
        is_default: false,
    },
    TemplateMeta {
        id: "report/academic",
        profile: Profile::Report,
        title: "Academic",
        description: "学术论文排版",
        css_rel: "report/academic.css",
        sample_rel: None,
        is_default: false,
    },
    TemplateMeta {
        id: "report/business",
        profile: Profile::Report,
        title: "Business",
        description: "商务封面与强调色",
        css_rel: "report/business.css",
        sample_rel: None,
        is_default: false,
    },
    TemplateMeta {
        id: "report/narrow",
        profile: Profile::Report,
        title: "Narrow",
        description: "窄栏阅读布局",
        css_rel: "report/narrow.css",
        sample_rel: None,
        is_default: false,
    },
    TemplateMeta {
        id: "report/data",
        profile: Profile::Report,
        title: "Data",
        description: "数据报告表格强调",
        css_rel: "report/data.css",
        sample_rel: None,
        is_default: false,
    },
    TemplateMeta {
        id: "resume/classic",
        profile: Profile::Resume,
        title: "Classic",
        description: "经典单栏简历",
        css_rel: "resume/classic.css",
        sample_rel: Some("samples/resume-classic.md"),
        is_default: true,
    },
    TemplateMeta {
        id: "resume/modern",
        profile: Profile::Resume,
        title: "Modern",
        description: "现代深色页眉",
        css_rel: "resume/modern.css",
        sample_rel: None,
        is_default: false,
    },
    TemplateMeta {
        id: "resume/two-col",
        profile: Profile::Resume,
        title: "Two Column",
        description: "侧栏 + 正文双栏",
        css_rel: "resume/two-col.css",
        sample_rel: None,
        is_default: false,
    },
    TemplateMeta {
        id: "resume/compact",
        profile: Profile::Resume,
        title: "Compact",
        description: "紧凑一页简历",
        css_rel: "resume/compact.css",
        sample_rel: None,
        is_default: false,
    },
    TemplateMeta {
        id: "resume/even",
        profile: Profile::Resume,
        title: "Even Split",
        description: "左右均分区块",
        css_rel: "resume/even.css",
        sample_rel: None,
        is_default: false,
    },
];

const FILES: &[TemplateFile] = &[
    TemplateFile {
        rel_path: "slide/default.css",
        source: SLIDE_DEFAULT,
    },
    TemplateFile {
        rel_path: "slide/gaia.css",
        source: SLIDE_GAIA,
    },
    TemplateFile {
        rel_path: "slide/uncover.css",
        source: SLIDE_UNCOVER,
    },
    TemplateFile {
        rel_path: "slide/corporate.css",
        source: SLIDE_CORPORATE,
    },
    TemplateFile {
        rel_path: "slide/academic.css",
        source: SLIDE_ACADEMIC,
    },
    TemplateFile {
        rel_path: "slide/minimal-dark.css",
        source: SLIDE_MINIMAL_DARK,
    },
    TemplateFile {
        rel_path: "report/github-light.css",
        source: REPORT_GITHUB_LIGHT,
    },
    TemplateFile {
        rel_path: "report/github-dark.css",
        source: REPORT_GITHUB_DARK,
    },
    TemplateFile {
        rel_path: "report/academic.css",
        source: REPORT_ACADEMIC,
    },
    TemplateFile {
        rel_path: "report/business.css",
        source: REPORT_BUSINESS,
    },
    TemplateFile {
        rel_path: "report/narrow.css",
        source: REPORT_NARROW,
    },
    TemplateFile {
        rel_path: "report/data.css",
        source: REPORT_DATA,
    },
    TemplateFile {
        rel_path: "resume/classic.css",
        source: RESUME_CLASSIC,
    },
    TemplateFile {
        rel_path: "resume/modern.css",
        source: RESUME_MODERN,
    },
    TemplateFile {
        rel_path: "resume/two-col.css",
        source: RESUME_TWO_COL,
    },
    TemplateFile {
        rel_path: "resume/compact.css",
        source: RESUME_COMPACT,
    },
    TemplateFile {
        rel_path: "resume/even.css",
        source: RESUME_EVEN,
    },
    TemplateFile {
        rel_path: "samples/slide-default.md",
        source: SAMPLE_SLIDE,
    },
    TemplateFile {
        rel_path: "samples/report-github-light.md",
        source: SAMPLE_REPORT,
    },
    TemplateFile {
        rel_path: "samples/resume-classic.md",
        source: SAMPLE_RESUME,
    },
];

pub fn find_by_id(id: &str) -> Option<&'static TemplateMeta> {
    LISTABLE.iter().find(|m| m.id == id)
}

pub fn default_for_profile(profile: Profile) -> Option<&'static TemplateMeta> {
    LISTABLE
        .iter()
        .find(|m| m.profile == profile && m.is_default)
}

pub fn ids_for_profile(profile: Profile) -> Vec<&'static str> {
    LISTABLE
        .iter()
        .filter(|m| m.profile == profile)
        .map(|m| m.id)
        .collect()
}

pub fn sample_for_template(id: &str) -> Option<&'static str> {
    let meta = find_by_id(id)?;
    if let Some(rel) = meta.sample_rel {
        return file_source(rel);
    }
    default_for_profile(meta.profile)
        .and_then(|d| d.sample_rel)
        .and_then(file_source)
}

fn file_source(rel_path: &str) -> Option<&'static str> {
    FILES
        .iter()
        .find(|f| f.rel_path == rel_path)
        .map(|f| f.source)
}

pub fn theme_css(meta: &TemplateMeta) -> String {
    let raw = FILES
        .iter()
        .find(|f| f.rel_path == meta.css_rel)
        .map(|f| f.source)
        .unwrap_or("");
    resolve_css_imports(raw)
}

fn strip_marp_theme_header(css: &str) -> String {
    css.lines()
        .filter(|line| !line.trim().starts_with("/* @theme"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn resolve_css_imports(raw: &str) -> String {
    let mut out = raw.to_string();
    for _ in 0..8 {
        let prev = out.clone();
        out = out.replace("@import \"../common/base.css\";", BASE_CSS);
        // Marpit themeSet.add() accepts one @theme header per CSS blob; strip
        // imported slide base theme header when inlining default.
        out = out.replace(
            "@import \"default\";",
            &strip_marp_theme_header(SLIDE_DEFAULT),
        );
        out = out.replace("@import \"github-light.css\";", REPORT_GITHUB_LIGHT);
        out = out.replace("@import \"github-light\";", REPORT_GITHUB_LIGHT);
        out = out.replace("@import \"classic.css\";", RESUME_CLASSIC);
        out = out.replace("@import \"classic\";", RESUME_CLASSIC);
        if out == prev {
            break;
        }
    }
    out.lines()
        .filter(|line| !line.trim().starts_with("@import"))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn list_ids() -> Vec<&'static str> {
    LISTABLE.iter().map(|m| m.id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_css_flattens_imports_for_business_and_github_light() {
        for id in ["report/business", "report/github-light", "resume/modern"] {
            let meta = find_by_id(id).unwrap();
            let css = theme_css(meta);
            assert!(!css.contains("@import"), "{id} theme still has @import");
            assert!(
                css.contains("border-collapse"),
                "{id} theme missing table rules"
            );
        }
    }

    #[test]
    fn slide_theme_css_keeps_single_marp_theme_header() {
        for id in ["slide/gaia", "slide/uncover", "slide/corporate"] {
            let meta = find_by_id(id).unwrap();
            let css = theme_css(meta);
            assert_eq!(
                css.matches("/* @theme").count(),
                1,
                "{id} must expose exactly one @theme header for Marpit"
            );
            assert!(
                css.contains("linear-gradient")
                    || css.contains("#111827")
                    || css.contains("#0f766e"),
                "{id} missing visual theme rules"
            );
        }
    }
}
