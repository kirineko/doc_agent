import { describe, expect, it } from "vitest";
import {
  formatFileBusyMessage,
  formatToolResultError,
  isFileBusySummary,
  parseFileBusyError,
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
    ).toBe("path escapes project");
  });
});
