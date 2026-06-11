import { describe, expect, it } from "vitest";
import { normalizeBriefEntries } from "./clarifyBrief";

describe("normalizeBriefEntries", () => {
  it("unwraps single nested wrapper object", () => {
    const entries = normalizeBriefEntries({
      创作简报: {
        文档类型: "PPT",
        "主题/目标": "年度总结",
      },
    });
    expect(entries).toEqual([
      ["文档类型", "PPT"],
      ["主题/目标", "年度总结"],
    ]);
  });

  it("keeps flat brief as-is", () => {
    const entries = normalizeBriefEntries({
      文档类型: "Word",
      排版风格: "**简约留白**",
    });
    expect(entries).toEqual([
      ["文档类型", "Word"],
      ["排版风格", "**简约留白**"],
    ]);
  });

  it("flattens nested brief field values", () => {
    const entries = normalizeBriefEntries({
      创作简报: {
        文档类型: "PPT",
        样式要点: { 字体: "微软雅黑", 配色: "科技蓝" },
      },
    });
    expect(entries).toEqual([
      ["文档类型", "PPT"],
      ["样式要点", "字体：微软雅黑\n配色：科技蓝"],
    ]);
  });
});
