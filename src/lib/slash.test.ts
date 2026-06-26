import { describe, expect, it } from "vitest";
import { applySlash, applySlashCommand, buildSlashInputFromPalette, detectSlash } from "./slash";
import { flattenSlashGroups, searchSlashCommands } from "./slashFuzzy";
import { CATEGORY_ORDER, SLASH_COMMANDS } from "./slashCommands";

describe("slash", () => {
  it("detects / at line start", () => {
    const state = detectSlash("/word", 5);
    expect(state?.query).toBe("word");
    expect(state?.start).toBe(0);
  });

  it("detects / after whitespace", () => {
    const state = detectSlash("你好 /read", 8);
    expect(state?.query).toBe("read");
    expect(state?.start).toBe(3);
  });

  it("detects full command id with colon separator", () => {
    const state = detectSlash("/word:edit", 11);
    expect(state?.query).toBe("word:edit");
    expect(state?.start).toBe(0);
  });

  it("ignores / in the middle of a word", () => {
    expect(detectSlash("foo/bar", 7)).toBeNull();
  });

  it("ignores / when query contains space", () => {
    expect(detectSlash("/word edit", 6)).toBeNull();
  });

  it("ends slash detection after command pick with trailing space", () => {
    const state = detectSlash("/comp", 5)!;
    const result = applySlashCommand("/comp", state, "compact", true);
    expect(result.text).toBe("/compact ");
    expect(detectSlash(result.text, result.cursor)).toBeNull();
  });

  it("replaces /query with prompt and selects first placeholder", () => {
    const state = detectSlash("/read", 5)!;
    const prompt = "请阅读 {{文件名}}，概括。";
    const result = applySlash("/read", state, prompt);
    expect(result.text).toBe(prompt);
    expect(result.cursor).toBe(prompt.indexOf("{{"));
    expect(result.selectionEnd).toBe(prompt.indexOf("}}") + 2);
  });

  it("appends palette slash commands without wiping existing draft", () => {
    const init = SLASH_COMMANDS.find((command) => command.id === "init");
    expect(init).toBeDefined();
    const result = buildSlashInputFromPalette("已有草稿", init!);
    expect(result.text).toBe("已有草稿 /init ");
    expect(result.cursor).toBe(result.text.length);
  });
});

describe("slashFuzzy", () => {
  it("returns 30 commands grouped in category order", () => {
    const groups = searchSlashCommands("");
    const flat = flattenSlashGroups(groups);
    expect(flat).toHaveLength(30);
    expect(groups.map((g) => g.category)).toEqual(CATEGORY_ORDER);
  });

  it("has a single read command without category prefix", () => {
    const readCommands = SLASH_COMMANDS.filter((c) => c.id === "read" || c.id.endsWith(":read"));
    expect(readCommands).toHaveLength(1);
    expect(readCommands[0]?.id).toBe("read");
  });

  it("filters by word query", () => {
    const flat = flattenSlashGroups(searchSlashCommands("word"));
    expect(flat.some((m) => m.command.id === "word:create")).toBe(true);
    expect(flat.some((m) => m.command.category === "word")).toBe(true);
  });

  it("matches 批注 to word:comment", () => {
    const flat = flattenSlashGroups(searchSlashCommands("批注"));
    expect(flat.some((m) => m.command.id === "word:comment")).toBe(true);
  });

  it("matches markdown query to markdown templates", () => {
    const flat = flattenSlashGroups(searchSlashCommands("markdown"));
    expect(flat.some((m) => m.command.id === "markdown:slide")).toBe(true);
    expect(flat.some((m) => m.command.id === "markdown:report")).toBe(true);
    expect(flat.filter((m) => m.command.category === "markdown")).toHaveLength(4);
  });

  it("matches 下载 to download-images", () => {
    const flat = flattenSlashGroups(searchSlashCommands("下载"));
    expect(flat.some((m) => m.command.id === "download-images")).toBe(true);
  });
});

describe("slashCommands registry", () => {
  it("includes init in command group before general", () => {
    const groups = searchSlashCommands("");
    expect(groups[0]?.category).toBe("command");
    expect(groups.map((g) => g.category)).toEqual(CATEGORY_ORDER);
    const init = flattenSlashGroups(searchSlashCommands("init")).find(
      (m) => m.command.id === "init",
    );
    expect(init?.command.category).toBe("command");
    expect(init?.command.kind).toBe("command");
  });

  it("matches ppt:edit-ooxml by ooxml keyword", () => {
    const flat = flattenSlashGroups(searchSlashCommands("ooxml"));
    expect(flat.some((m) => m.command.id === "ppt:edit-ooxml")).toBe(true);
  });

  it("includes compact command in command group", () => {
    const compact = flattenSlashGroups(searchSlashCommands("compact")).find(
      (m) => m.command.id === "compact",
    );
    expect(compact?.command.category).toBe("command");
    expect(compact?.command.kind).toBe("command");
    if (compact?.command.kind === "command") {
      expect(compact.command.acceptsTail).toBeFalsy();
    }
  });

  it("has 28 template entries", () => {
    expect(SLASH_COMMANDS.filter((c) => c.kind === "template")).toHaveLength(28);
  });

  it("includes markdown category with four templates", () => {
    const markdown = SLASH_COMMANDS.filter((c) => c.category === "markdown");
    expect(markdown).toHaveLength(4);
    expect(markdown.map((c) => c.id)).toEqual([
      "markdown:slide",
      "markdown:report",
      "markdown:resume",
      "markdown:convert",
    ]);
  });

  it("download-images prompt uses theme placeholder only", () => {
    const cmd = SLASH_COMMANDS.find((c) => c.id === "download-images");
    expect(cmd?.kind).toBe("template");
    if (cmd?.kind !== "template") return;
    expect(cmd.prompt).toContain("{{主题}}");
    expect(cmd.prompt).not.toMatch(/URL/i);
  });

  it("markdown prompts use 转 HTML not tool names", () => {
    for (const id of ["markdown:slide", "markdown:report", "markdown:resume"]) {
      const cmd = SLASH_COMMANDS.find((c) => c.id === id);
      expect(cmd?.kind).toBe("template");
      if (cmd?.kind !== "template") continue;
      expect(cmd.prompt).toMatch(/转 HTML/);
      expect(cmd.prompt).not.toContain("markdown_to_html");
    }
  });

  it("web:report description distinguishes from markdown templates", () => {
    const webReport = SLASH_COMMANDS.find((c) => c.id === "web:report");
    expect(webReport?.description).toContain("非 Markdown 模板");
  });

  it("keeps template prompts within 20–100 characters", () => {
    // ppt:edit-ooxml must enumerate the full OOXML tool chain
    // (ooxml_unpack → slide XML → ooxml_pack) plus forbidden tools, so it is
    // allowed a higher ceiling than the generic prompts.
    const maxLenById: Record<string, number> = { "ppt:edit-ooxml": 130 };
    for (const command of SLASH_COMMANDS) {
      if (command.kind !== "template") continue;
      const len = command.prompt.length;
      expect(len, `${command.id} prompt length ${len}`).toBeGreaterThanOrEqual(20);
      expect(len, `${command.id} prompt length ${len}`).toBeLessThanOrEqual(
        maxLenById[command.id] ?? 100,
      );
    }
  });
});
