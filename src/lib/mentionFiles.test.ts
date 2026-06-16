import { describe, expect, it } from "vitest";
import {
  flattenMentionFileGroups,
  groupMentionFileMatches,
  highlightBasenamePositions,
  orderMentionFileMatchesForDisplay,
  parseMentionBrowseContext,
  searchMentionFiles,
} from "./mentionFiles";
import type { MentionFileEntry } from "./projectFiles";

describe("searchMentionFiles", () => {
  const entries: MentionFileEntry[] = [
    { path: "docs", isDir: true, modifiedMs: 300 },
    { path: "docs/report.docx", isDir: false, modifiedMs: 200 },
    { path: "docs/assets", isDir: true, modifiedMs: 150 },
    { path: "docs/assets/logo.png", isDir: false, modifiedMs: 140 },
    { path: "readme.txt", isDir: false, modifiedMs: 100 },
  ];

  it("returns root entries only when query empty", () => {
    const results = searchMentionFiles("", entries);
    expect(results.map((item) => item.item)).toEqual(["docs", "readme.txt"]);
    expect(results[0]?.isDir).toBe(true);
  });

  it("lists direct children when browsing a directory", () => {
    const results = searchMentionFiles("docs/", entries);
    expect(results.map((item) => item.item)).toEqual(["docs/assets", "docs/report.docx"]);
  });

  it("does not match every child when scoped term matches basename only", () => {
    const results = searchMentionFiles("docs/d", entries);
    expect(results.map((item) => item.item)).toEqual(["docs/report.docx"]);
  });

  it("filters within directory scope", () => {
    const results = searchMentionFiles("docs/rep", entries);
    expect(results.map((item) => item.item)).toEqual(["docs/report.docx"]);
  });

  it("groups global matches by parent directory", () => {
    const results = searchMentionFiles("logo", entries);
    const groups = groupMentionFileMatches(results, parseMentionBrowseContext("logo"));
    expect(groups).toHaveLength(1);
    expect(groups[0]?.label).toBe("docs/assets/");
    expect(groups[0]?.items[0]?.basename).toBe("logo.png");
  });

  it("filters without truncating to eight", () => {
    const many = Array.from({ length: 12 }, (_, index) => ({
      path: `file-${index}.txt`,
      isDir: false,
      modifiedMs: index,
    }));
    expect(searchMentionFiles("file", many)).toHaveLength(12);
  });

  it("maps highlight positions to basename", () => {
    const match = searchMentionFiles("rep", entries)[0]!;
    expect(match?.item).toBe("docs/report.docx");
    expect(highlightBasenamePositions(match)).toEqual([0, 1, 2]);
  });

  it("orders display matches the same as grouped popup flattening", () => {
    const mixed: MentionFileEntry[] = [
      { path: "b/note.txt", isDir: false, modifiedMs: 200 },
      { path: "a/note.txt", isDir: false, modifiedMs: 100 },
    ];
    const results = searchMentionFiles("note", mixed);
    const ctx = parseMentionBrowseContext("note");
    const grouped = groupMentionFileMatches(results, ctx);
    const ordered = orderMentionFileMatchesForDisplay(results, "note");
    expect(ordered.map((item) => item.item)).toEqual(
      flattenMentionFileGroups(grouped).map((item) => item.item),
    );
    expect(ordered.map((item) => item.item)).toEqual(["a/note.txt", "b/note.txt"]);
  });
});
