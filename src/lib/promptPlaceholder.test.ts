import { describe, expect, it } from "vitest";
import {
  deletePlaceholderAtCursor,
  deletePlaceholderBeforeCursor,
  findPlaceholders,
  firstPlaceholder,
} from "./promptPlaceholder";

describe("promptPlaceholder", () => {
  it("finds placeholders", () => {
    const text = "主题是{{主题}}，页数{{页数}}";
    expect(findPlaceholders(text)).toEqual([
      { start: 3, end: 9, hint: "主题" },
      { start: 12, end: 18, hint: "页数" },
    ]);
  });

  it("firstPlaceholder respects offset", () => {
    const text = "主题是{{主题}}，页数{{页数}}";
    expect(firstPlaceholder(text, 10)?.hint).toBe("页数");
  });

  it("deletes placeholder on backspace inside", () => {
    const text = "修改 {{文件名.docx}} 内容";
    const cursor = text.indexOf("文件名");
    const result = deletePlaceholderBeforeCursor(text, cursor);
    expect(result?.text).toBe("修改  内容");
    expect(result?.cursor).toBe(3);
  });

  it("deletes placeholder on delete at start", () => {
    const text = "修改 {{文件名.docx}} 内容";
    const start = text.indexOf("{{");
    const result = deletePlaceholderAtCursor(text, start);
    expect(result?.text).toBe("修改  内容");
  });
});
