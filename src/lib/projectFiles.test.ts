import { describe, expect, it } from "vitest";
import {
  isIgnoredMentionPath,
  mergeProjectFileEntries,
  sameMentionFileEntries,
  sortMentionFileEntries,
} from "./projectFiles";

describe("mergeProjectFileEntries", () => {
  it("merges new paths and dedupes", () => {
    expect(
      mergeProjectFileEntries(
        [{ path: "a.md", isDir: false, modifiedMs: 1 }],
        ["b.md", "a.md"],
      ).map((entry) => entry.path),
    ).toEqual(["b.md", "a.md"]);
  });

  it("skips ooxml unpack internals", () => {
    expect(
      mergeProjectFileEntries([], ["unpacked/word/document.xml", "report.docx"]).map(
        (entry) => entry.path,
      ),
    ).toEqual(["report.docx"]);
  });
});

describe("sortMentionFileEntries", () => {
  it("sorts by modified time descending", () => {
    const sorted = sortMentionFileEntries([
      { path: "old.md", isDir: false, modifiedMs: 1 },
      { path: "new.md", isDir: false, modifiedMs: 99 },
    ]);
    expect(sorted[0]?.path).toBe("new.md");
  });
});

describe("sameMentionFileEntries", () => {
  it("compares ordered entries", () => {
    const a = [{ path: "a", isDir: true, modifiedMs: 1 }];
    expect(sameMentionFileEntries(a, [...a])).toBe(true);
    expect(sameMentionFileEntries(a, [{ path: "b", isDir: false, modifiedMs: 1 }])).toBe(false);
  });
});

describe("isIgnoredMentionPath", () => {
  it("detects unpacked work dirs", () => {
    expect(isIgnoredMentionPath("unpacked/word/document.xml")).toBe(true);
    expect(isIgnoredMentionPath("contract_unpacked/word/document.xml")).toBe(true);
    expect(isIgnoredMentionPath("report.docx")).toBe(false);
  });
});
