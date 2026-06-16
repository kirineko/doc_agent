import { describe, expect, it } from "vitest";
import {
  basenameFromPath,
  buildMentionInsert,
  conflictStrategyForAction,
  isImportFileExistsError,
  mapConflictDialogResult,
} from "./projectImport";
import { insertSlashPrompt } from "./slash";

describe("projectImport", () => {
  it("buildMentionInsert inserts at cursor with trailing space", () => {
    const result = buildMentionInsert("请分析 |", 4, ["a.docx", "b.pdf"]);
    expect(result.text).toBe("请分析 @a.docx @b.pdf |");
    expect(result.cursor).toBe("请分析 @a.docx @b.pdf ".length);
  });

  it("quotes paths with spaces", () => {
    const result = buildMentionInsert("", 0, ["my report.docx"]);
    expect(result.text).toBe('@"my report.docx" ');
  });

  it("detects file exists error", () => {
    expect(isImportFileExistsError(new Error("file already exists: a.txt"))).toBe(true);
    expect(isImportFileExistsError("other")).toBe(false);
  });

  it("maps conflict dialog labels", () => {
    expect(
      mapConflictDialogResult("覆盖", { overwrite: "覆盖", rename: "另存为" }),
    ).toBe("overwrite");
    expect(mapConflictDialogResult("另存为", { overwrite: "覆盖", rename: "另存为" })).toBe(
      "rename",
    );
    expect(mapConflictDialogResult(null, { overwrite: "覆盖", rename: "另存为" })).toBe("cancel");
  });

  it("maps conflict strategy", () => {
    expect(conflictStrategyForAction("overwrite")).toBe("overwrite");
    expect(conflictStrategyForAction("rename")).toBe("rename");
    expect(conflictStrategyForAction("cancel")).toBeNull();
  });

  it("extracts basename", () => {
    expect(basenameFromPath("/tmp/foo/bar.docx")).toBe("bar.docx");
    expect(basenameFromPath("note.txt")).toBe("note.txt");
  });
});

describe("insertSlashPrompt", () => {
  it("inserts prompt at cursor when no slash state", () => {
    const prompt = "请阅读 {{文件名}}，概括。";
    const result = insertSlashPrompt("你好", 2, prompt);
    expect(result.text).toBe(`你好${prompt}`);
    expect(result.cursor).toBe(2 + prompt.indexOf("{{"));
  });

  it("replaces active slash query", () => {
    const prompt = "请阅读 {{文件名}}，概括。";
    const result = insertSlashPrompt("/read", 5, prompt);
    expect(result.text).toBe(prompt);
  });
});
