import { describe, expect, it } from "vitest";
import { fuzzyMatch } from "./fuzzy";

describe("fuzzyMatch", () => {
  const items = ["课程体系.xlsx", "课程大纲-AI辅助.docx", "资料/汇报.pptx"];

  it("ranks Chinese filename matches", () => {
    const results = fuzzyMatch("课程", items);
    expect(results.length).toBeGreaterThan(0);
    expect(results[0]?.item).toContain("课程");
  });

  it("matches path segments", () => {
    const results = fuzzyMatch("汇报", items);
    expect(results.some((r) => r.item.includes("汇报.pptx"))).toBe(true);
  });

  it("sorts better matches first", () => {
    const results = fuzzyMatch("课程体", items);
    expect(results[0]?.item).toBe("课程体系.xlsx");
  });
});
