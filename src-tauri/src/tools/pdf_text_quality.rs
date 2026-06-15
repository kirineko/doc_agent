//! PDFium 文本质量评估：规范化、硬规则、按页 suspicion、代表页选取。

const MIN_PAGE_CHARS: u32 = 80;
const SUSPICION_THRESHOLD: f32 = 0.35;
const CID_MIN_COUNT: usize = 3;

#[derive(Debug, Clone, PartialEq)]
pub struct PageTextStats {
    pub index: u32,
    pub char_count: u32,
    pub suspicion: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SamplePagePick {
    pub index: u32,
    pub reason: &'static str,
}

/// 清理单页 PDFium 文本：去首尾空白、统一换行、合并行内空白、折叠多余空行。
pub fn normalize_page_text(text: &str) -> String {
    let mut lines = Vec::new();
    let mut prev_empty = false;
    for line in text.replace('\r', "").lines() {
        let trimmed = line.trim();
        let empty = trimmed.is_empty();
        if empty && prev_empty {
            continue;
        }
        if empty {
            lines.push(String::new());
        } else {
            lines.push(collapse_inline_whitespace(trimmed));
        }
        prev_empty = empty;
    }
    while lines.first().is_some_and(|l| l.is_empty()) {
        lines.remove(0);
    }
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    lines.join("\n")
}

fn collapse_inline_whitespace(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut prev_space = false;
    for c in line.chars() {
        if c.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    out
}

/// 将多页文本规范后拼接：跳过空白页，页间用双换行分隔（利于 Markdown 阅读）。
pub fn format_extracted_text(pages: &[(u32, &str)]) -> String {
    pages
        .iter()
        .map(|(_, text)| normalize_page_text(text))
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// 兼容旧调用名。
pub fn join_page_texts(pages: &[(u32, String)]) -> String {
    format_extracted_text(
        &pages
            .iter()
            .map(|(i, t)| (*i, t.as_str()))
            .collect::<Vec<_>>(),
    )
}

/// 按页计算 suspicion（0..1），越高越可能需要 vision。
pub fn page_suspicion(text: &str) -> f32 {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return 0.0;
    }
    let len = trimmed.chars().count().max(1) as f32;
    let mut score = 0.0f32;

    let cid_count = trimmed.matches("(cid:").count() as f32;
    score += (cid_count / len * 200.0).min(0.5);

    let replacement = trimmed
        .chars()
        .filter(|c| *c == '\u{FFFD}' || *c == '□')
        .count() as f32;
    score += (replacement / len * 100.0).min(0.4);

    let math_chars = trimmed
        .chars()
        .filter(|c| {
            matches!(
                c,
                '∫' | '∑' | '√' | '∞' | '±' | '≤' | '≥' | 'π' | 'α' | 'β' | 'θ'
            )
        })
        .count() as f32;
    score += (math_chars / len * 50.0).min(0.35);

    // PDFium 公式碎片：单字符“词”占比高
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    if !tokens.is_empty() {
        let short = tokens.iter().filter(|t| t.chars().count() <= 2).count() as f32;
        score += (short / tokens.len() as f32 * 0.25).min(0.25);
    }

    score.clamp(0.0, 1.0)
}

pub fn build_page_stats(pages: &[(u32, String)]) -> Vec<PageTextStats> {
    pages
        .iter()
        .map(|(index, text)| {
            let normalized = normalize_page_text(text);
            PageTextStats {
                index: *index,
                char_count: normalized.chars().count() as u32,
                suspicion: page_suspicion(&normalized),
            }
        })
        .collect()
}

/// 全文硬规则：命中则直接全量 vision，跳过 Judge。
pub fn full_text_hard_rule(full_text: &str, page_count: u32) -> Option<&'static str> {
    let trimmed = full_text.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.matches("(cid:").count() >= CID_MIN_COUNT {
        return Some("cid_glyphs");
    }
    let len = trimmed.chars().count();
    let replacement = trimmed
        .chars()
        .filter(|c| *c == '\u{FFFD}' || *c == '□')
        .count();
    if len > 0 && replacement * 100 / len.max(1) >= 5 {
        return Some("replacement_chars");
    }
    if len < 50 && page_count >= 1 {
        return Some("suspiciously_short");
    }
    None
}

pub fn pick_sample_page(stats: &[PageTextStats]) -> Option<SamplePagePick> {
    if stats.is_empty() {
        return None;
    }
    if stats.len() == 1 {
        return Some(SamplePagePick {
            index: stats[0].index,
            reason: "single_page",
        });
    }

    if let Some(best) = stats
        .iter()
        .filter(|p| p.suspicion >= SUSPICION_THRESHOLD)
        .max_by(|a, b| {
            a.suspicion
                .partial_cmp(&b.suspicion)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.index.cmp(&a.index))
        })
    {
        return Some(SamplePagePick {
            index: best.index,
            reason: "max_suspicion",
        });
    }

    let first = &stats[0];
    if first.char_count < MIN_PAGE_CHARS {
        let middle_pos = stats.len() / 2;
        let middle = stats.get(middle_pos);
        if let Some(mid) = middle {
            if mid.char_count >= MIN_PAGE_CHARS {
                return Some(SamplePagePick {
                    index: mid.index,
                    reason: "sparse_first_page_middle",
                });
            }
        }
        if let Some(max_page) = stats.iter().max_by_key(|p| p.char_count) {
            if max_page.char_count > first.char_count {
                return Some(SamplePagePick {
                    index: max_page.index,
                    reason: "sparse_first_page_max_chars",
                });
            }
        }
    }

    Some(SamplePagePick {
        index: stats[0].index,
        reason: "default_first",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_page_text_collapses_inline_whitespace() {
        assert_eq!(
            normalize_page_text("hello   world\n\n  foo  "),
            "hello world\n\nfoo"
        );
    }

    #[test]
    fn format_extracted_text_skips_blank_pages() {
        let pages = vec![(1, "Cover"), (2, "   "), (3, "Body text")];
        assert_eq!(format_extracted_text(&pages), "Cover\n\nBody text");
    }

    #[test]
    fn pick_sample_single_page() {
        let stats = vec![PageTextStats {
            index: 1,
            char_count: 200,
            suspicion: 0.1,
        }];
        assert_eq!(pick_sample_page(&stats).unwrap().index, 1);
        assert_eq!(pick_sample_page(&stats).unwrap().reason, "single_page");
    }

    #[test]
    fn pick_sample_max_suspicion() {
        let stats = vec![
            PageTextStats {
                index: 1,
                char_count: 20,
                suspicion: 0.1,
            },
            PageTextStats {
                index: 2,
                char_count: 400,
                suspicion: 0.6,
            },
        ];
        let pick = pick_sample_page(&stats).unwrap();
        assert_eq!(pick.index, 2);
        assert_eq!(pick.reason, "max_suspicion");
    }

    #[test]
    fn pick_sample_sparse_cover_uses_middle_entry_in_range() {
        let stats = vec![
            PageTextStats {
                index: 10,
                char_count: 10,
                suspicion: 0.05,
            },
            PageTextStats {
                index: 11,
                char_count: 5,
                suspicion: 0.0,
            },
            PageTextStats {
                index: 12,
                char_count: 500,
                suspicion: 0.1,
            },
        ];
        let pick = pick_sample_page(&stats).unwrap();
        assert_eq!(pick.index, 12);
    }

    #[test]
    fn pick_sample_sparse_cover_uses_middle() {
        let stats = vec![
            PageTextStats {
                index: 1,
                char_count: 10,
                suspicion: 0.05,
            },
            PageTextStats {
                index: 2,
                char_count: 5,
                suspicion: 0.0,
            },
            PageTextStats {
                index: 3,
                char_count: 500,
                suspicion: 0.1,
            },
        ];
        let pick = pick_sample_page(&stats).unwrap();
        assert_eq!(pick.index, 3);
        assert!(pick.reason.contains("sparse_first_page"));
    }

    #[test]
    fn hard_rule_cid() {
        assert_eq!(
            full_text_hard_rule("foo (cid:1) bar (cid:2) baz (cid:3)", 2),
            Some("cid_glyphs")
        );
    }
}
