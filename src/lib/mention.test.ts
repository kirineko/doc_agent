import { describe, expect, it } from "vitest";
import {
  applyMention,
  cancelMentionAtState,
  deleteMentionBeforeCursor,
  detectMention,
  expandMentionDirectory,
  formatMentionPath,
  parseMentionTokenAt,
} from "./mention";

describe("mention", () => {
  it("detects @ before cursor without spaces", () => {
    const state = detectMention("分析 @课程", 6);
    expect(state?.query).toBe("课程");
    expect(state?.start).toBe(3);
  });

  it("ignores @ when query contains space", () => {
    expect(detectMention("分析 @课 程", 7)).toBeNull();
  });

  it("replaces mention with path", () => {
    const state = detectMention("分析 @课程", 6)!;
    const result = applyMention("分析 @课程", state, "课程体系.xlsx");
    expect(result.text).toBe("分析 @课程体系.xlsx ");
    expect(result.cursor).toBe(result.text.length);
  });

  it("expands directory mention for tab navigation", () => {
    const text = "分析 @doc";
    const state = detectMention(text, text.length)!;
    const result = expandMentionDirectory(text, state, "docs");
    expect(result.text).toBe("分析 @docs/");
    expect(result.cursor).toBe(result.text.length);
    expect(detectMention(result.text, result.cursor)?.query).toBe("docs/");
  });

  it("quotes paths with spaces", () => {
    expect(formatMentionPath("my report.docx")).toBe('"my report.docx"');
    const text = "分析 @rep";
    const state = detectMention(text, text.length)!;
    const result = applyMention(text, state, "my report.docx");
    expect(result.text).toBe('分析 @"my report.docx" ');
  });

  it("quotes paths with parser-stop punctuation", () => {
    expect(formatMentionPath("report(1).docx")).toBe('"report(1).docx"');
    expect(formatMentionPath("客户：Q2.docx")).toBe('"客户：Q2.docx"');
    const text = "分析 @rep";
    const state = detectMention(text, text.length)!;
    const result = applyMention(text, state, "report(1).docx");
    expect(result.text).toBe('分析 @"report(1).docx" ');
    const token = parseMentionTokenAt(result.text, result.text.indexOf("@"));
    expect(token?.path).toBe("report(1).docx");
  });

  it("parses quoted mention token", () => {
    const text = '分析 @"my report.docx" 继续';
    const token = parseMentionTokenAt(text, 3);
    expect(token?.path).toBe("my report.docx");
    expect(token?.end).toBe('分析 @"my report.docx"'.length);
  });

  it("deletes entire @path token on backspace inside path", () => {
    const text = "分析 @课程体系.xlsx 内容";
    const cursor = "分析 @课程体系.x".length;
    const result = deleteMentionBeforeCursor(text, cursor);
    expect(result?.text).toBe("分析  内容");
    expect(result?.cursor).toBe(3);
  });

  it("deletes @path and trailing space when backspace on delimiter space", () => {
    const text = "分析 @课程体系.xlsx 内容";
    const cursor = "分析 @课程体系.xlsx ".length;
    const result = deleteMentionBeforeCursor(text, cursor);
    expect(result?.text).toBe("分析 内容");
    expect(result?.cursor).toBe(3);
  });

  it("deletes quoted path with spaces without eating following text", () => {
    const text = '分析 @"my report.docx" 后续';
    const cursor = '分析 @"my report.docx"'.length;
    const result = deleteMentionBeforeCursor(text, cursor);
    expect(result?.text).toBe("分析  后续");
  });

  it("does not delete glued text after unquoted path", () => {
    const text = "分析 @report.docx后续";
    const cursor = "分析 @report.docx".length;
    const result = deleteMentionBeforeCursor(text, cursor);
    expect(result?.text).toBe("分析 后续");
  });

  it("deletes lone @ before full-width punctuation", () => {
    const text = "请阅读 @，概括内容结构；若是表格则总结关键数据，并给改进建议。";
    const cursor = "请阅读 @".length;
    const result = deleteMentionBeforeCursor(text, cursor);
    expect(result?.text).toBe(
      "请阅读 ，概括内容结构；若是表格则总结关键数据，并给改进建议。",
    );
    expect(result?.cursor).toBe("请阅读 ".length);
  });

  it("does not treat text after @， as mention query", () => {
    const text = "请阅读 @，概括内容";
    expect(detectMention(text, text.length)).toBeNull();
    expect(parseMentionTokenAt(text, text.indexOf("@"))?.end).toBe("请阅读 @".length);
  });

  it("cancels mention on escape by removing @ and query", () => {
    const text = "分析 @rep";
    const state = detectMention(text, text.length)!;
    const result = cancelMentionAtState(text, state);
    expect(result.text).toBe("分析 ");
    expect(result.cursor).toBe("分析 ".length);
  });

  it("cancels lone @ on escape", () => {
    const text = "分析 @";
    const state = detectMention(text, text.length)!;
    const result = cancelMentionAtState(text, state);
    expect(result.text).toBe("分析 ");
    expect(result.cursor).toBe("分析 ".length);
  });
});
