import { describe, expect, it } from "vitest";
import {
  formatFileBusyMessage,
  formatToolResultError,
  isFileBusySummary,
  parseFileBusyError,
  parseToolResultError,
} from "./fileBusy";

describe("parseFileBusyError", () => {
  it("parses backend file_busy JSON", () => {
    expect(
      parseFileBusyError(
        JSON.stringify({
          error: "file_busy",
          path: "report.docx",
          message: "文件 report.docx 已被会话 周报 占用",
          blocking_session_id: "sess-1",
        }),
      ),
    ).toEqual({
      error: "file_busy",
      path: "report.docx",
      message: "文件 report.docx 已被会话 周报 占用",
      blocking_session_id: "sess-1",
    });
  });

  it("returns undefined for non-json or other errors", () => {
    expect(parseFileBusyError("permission denied")).toBeUndefined();
    expect(parseFileBusyError(JSON.stringify({ error: "other", path: "a" }))).toBeUndefined();
  });
});

describe("formatFileBusyMessage", () => {
  it("prefers backend message when present", () => {
    const parsed = parseFileBusyError(
      JSON.stringify({ error: "file_busy", path: "a.docx", message: "自定义占用提示" }),
    );
    expect(parsed && formatFileBusyMessage(parsed)).toBe("自定义占用提示");
  });

  it("builds fallback from path and session title", () => {
    const parsed = parseFileBusyError(
      JSON.stringify({ error: "file_busy", path: "a.docx", blocking_session_id: "s1" }),
    );
    expect(parsed && formatFileBusyMessage(parsed, "周报整理")).toBe(
      "文件 a.docx 已被周报整理占用，请稍后重试。",
    );
  });
});

describe("formatToolResultError", () => {
  it("formats file_busy summary for tool chain display", () => {
    const summary = JSON.stringify({
      error: "file_busy",
      path: "report.docx",
      message: "当前 report.docx 已被会话「周报」占用，请稍后重试。",
      blocking_session_id: "sess-1",
    });
    expect(formatToolResultError(summary)).toBe(
      "当前 report.docx 已被会话「周报」占用，请稍后重试。",
    );
    expect(isFileBusySummary(summary)).toBe(true);
  });

  it("falls back to generic json error message", () => {
    expect(
      formatToolResultError(JSON.stringify({ error: "sandbox", message: "path escapes project" })),
    ).toBe("sandbox");
  });
});

describe("parseToolResultError", () => {
  it("parses timeout JSON with headline, detail, and hint", () => {
    const parsed = parseToolResultError(
      JSON.stringify({
        error: "script timeout",
        detail: "script timeout",
        hint: "脚本在 120s 内未完成…用 doc_image_resize 缩图",
      }),
    );
    expect(parsed.headline).toBe("script timeout");
    expect(parsed.detail).toBe("script timeout");
    expect(parsed.hint).toContain("doc_image_resize");
  });

  it("parses runtime error JSON with stack in detail", () => {
    const parsed = parseToolResultError(
      JSON.stringify({
        error: "JavaScript runtime error",
        detail: "TypeError: slide.addImage is not a function\n  at main (script.js:12)",
      }),
    );
    expect(parsed.headline).toBe("JavaScript runtime error");
    expect(parsed.detail).toContain("TypeError");
    expect(parsed.detail).toContain("script.js:12");
  });

  it("truncates long detail keeping the tail and falls back for non-JSON", () => {
    const head = "H".repeat(400);
    const mid = "M".repeat(50);
    const tail = "T".repeat(200);
    const parsed = parseToolResultError(
      JSON.stringify({
        error: "JavaScript runtime error",
        detail: `${head}${mid}${tail}`,
      }),
    );
    expect(parsed.detail).toBe(`${head}…${tail}`);
    expect(parsed.detail?.includes("M")).toBe(false);

    expect(parseToolResultError("permission denied")).toEqual({
      headline: "permission denied",
    });
  });
});
