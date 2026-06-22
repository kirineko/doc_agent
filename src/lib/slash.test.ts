import { describe, expect, it } from "vitest";
import { applySlash, detectSlash } from "./slash";
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

  it("replaces /query with prompt and selects first placeholder", () => {
    const state = detectSlash("/read", 5)!;
    const prompt = "请阅读 {{文件名}}，概括。";
    const result = applySlash("/read", state, prompt);
    expect(result.text).toBe(prompt);
    expect(result.cursor).toBe(prompt.indexOf("{{"));
    expect(result.selectionEnd).toBe(prompt.indexOf("}}") + 2);
  });
});

describe("slashFuzzy", () => {
  it("returns 23 commands grouped in category order", () => {
    const groups = searchSlashCommands("");
    const flat = flattenSlashGroups(groups);
    expect(flat).toHaveLength(23);
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

  it("keeps template prompts within 20–100 characters", () => {
    for (const command of SLASH_COMMANDS) {
      if (command.kind !== "template") continue;
      const len = command.prompt.length;
      expect(len, `${command.id} prompt length ${len}`).toBeGreaterThanOrEqual(20);
      expect(len, `${command.id} prompt length ${len}`).toBeLessThanOrEqual(100);
    }
  });
});
