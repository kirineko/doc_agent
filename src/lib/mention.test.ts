import { describe, expect, it } from "vitest";
import { applyMention, deleteMentionBeforeCursor, detectMention } from "./mention";

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

  it("deletes entire @path token on backspace inside path", () => {
    const text = "分析 @课程体系.xlsx 内容";
    const cursor = "分析 @课程体系.x".length;
    const result = deleteMentionBeforeCursor(text, cursor);
    expect(result?.text).toBe("分析 内容");
    expect(result?.cursor).toBe(3);
  });

  it("deletes @path and trailing space when backspace on space", () => {
    const text = "分析 @课程体系.xlsx 内容";
    const cursor = "分析 @课程体系.xlsx ".length;
    const result = deleteMentionBeforeCursor(text, cursor);
    expect(result?.text).toBe("分析 内容");
  });
});
