import { describe, expect, it } from "vitest";
import {
  isIgnoredMentionPath,
  mergeProjectFilePaths,
  sameStringArrays,
} from "./projectFiles";

describe("mergeProjectFilePaths", () => {
  it("merges new paths and dedupes", () => {
    expect(mergeProjectFilePaths(["a.md"], ["b.md", "a.md"])).toEqual(["a.md", "b.md"]);
  });

  it("skips ooxml unpack internals", () => {
    expect(
      mergeProjectFilePaths([], ["unpacked/word/document.xml", "report.docx"]),
    ).toEqual(["report.docx"]);
  });
});

describe("sameStringArrays", () => {
  it("compares ordered string arrays", () => {
    expect(sameStringArrays([], [])).toBe(true);
    expect(sameStringArrays(["a", "b"], ["a", "b"])).toBe(true);
    expect(sameStringArrays(["a", "b"], ["b", "a"])).toBe(false);
    expect(sameStringArrays(["a"], ["a", "b"])).toBe(false);
  });
});

describe("isIgnoredMentionPath", () => {
  it("detects unpacked work dirs", () => {
    expect(isIgnoredMentionPath("unpacked/word/document.xml")).toBe(true);
    expect(isIgnoredMentionPath("contract_unpacked/word/document.xml")).toBe(true);
    expect(isIgnoredMentionPath("report.docx")).toBe(false);
  });
});
