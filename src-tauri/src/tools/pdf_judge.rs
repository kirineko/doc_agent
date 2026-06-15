use crate::agent::types::ModelId;
use crate::tools::vision_subcall::vision_subcall;
use crate::tools::{ToolContext, ToolError};

const MAX_PAGE_TEXT_IN_PROMPT: usize = 4000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JudgeVerdict {
    TextOk,
    NeedVision,
}

pub fn parse_judge_verdict(raw: &str) -> JudgeVerdict {
    let mut saw_ok = false;
    let mut saw_vision = false;

    for line in raw.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let norm = line
            .trim_end_matches(|c: char| ".!?，。；;".contains(c))
            .to_ascii_uppercase();
        match norm.as_str() {
            "NEED_VISION" => saw_vision = true,
            "TEXT_OK" | "TEXT SUFFICIENT" => saw_ok = true,
            _ => {}
        }
    }

    if saw_vision {
        JudgeVerdict::NeedVision
    } else if saw_ok {
        JudgeVerdict::TextOk
    } else {
        JudgeVerdict::NeedVision
    }
}

pub async fn judge_page_compare(
    ctx: &ToolContext<'_>,
    model_id: ModelId,
    page_index: u32,
    image_rel_path: &str,
    page_text: &str,
) -> Result<JudgeVerdict, ToolError> {
    let snippet: String = page_text.chars().take(MAX_PAGE_TEXT_IN_PROMPT).collect();
    let prompt = format!(
        "你是 PDF 文本提取质量裁判。附件为 PDF 第 {page_index} 页渲染图；下方是同页 PDFium 提取文本。\n\
         请对比：文本是否忠实反映图中全部可见内容（公式、上下标、分式、表格版式）？\n\n\
         --- PDFium 文本（第 {page_index} 页）---\n{snippet}\n---\n\n\
         若图文一致且为普通叙述/说明，回复 TEXT_OK。\n\
         若公式/符号/版式明显丢失、错位、乱码，或图中有大量文本未出现在提取里，回复 NEED_VISION。\n\
         不确定时回复 NEED_VISION。\n\
         只输出 TEXT_OK 或 NEED_VISION。"
    );

    let raw = vision_subcall(ctx, model_id, &[image_rel_path.to_string()], &prompt).await?;
    Ok(parse_judge_verdict(&raw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_text_ok() {
        assert_eq!(parse_judge_verdict("TEXT_OK"), JudgeVerdict::TextOk);
        assert_eq!(parse_judge_verdict("text_ok"), JudgeVerdict::TextOk);
    }

    #[test]
    fn parse_need_vision() {
        assert_eq!(
            parse_judge_verdict("NEED_VISION"),
            JudgeVerdict::NeedVision
        );
    }

    #[test]
    fn parse_unknown_defaults_need_vision() {
        assert_eq!(parse_judge_verdict("maybe"), JudgeVerdict::NeedVision);
        assert_eq!(parse_judge_verdict(""), JudgeVerdict::NeedVision);
    }

    #[test]
    fn parse_prose_mentioning_both_defaults_need_vision() {
        assert_eq!(
            parse_judge_verdict("The answer is NEED_VISION, not TEXT_OK"),
            JudgeVerdict::NeedVision
        );
    }

    #[test]
    fn parse_verdict_before_explanation() {
        assert_eq!(
            parse_judge_verdict("TEXT_OK\n图文一致，普通叙述。"),
            JudgeVerdict::TextOk
        );
        assert_eq!(
            parse_judge_verdict("NEED_VISION\n公式明显失真。"),
            JudgeVerdict::NeedVision
        );
    }

    #[test]
    fn parse_last_line_wins() {
        assert_eq!(
            parse_judge_verdict("reasoning...\nNEED_VISION"),
            JudgeVerdict::NeedVision
        );
        assert_eq!(
            parse_judge_verdict("reasoning...\nTEXT_OK"),
            JudgeVerdict::TextOk
        );
    }
}
