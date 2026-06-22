import { describe, expect, it } from "vitest";
import {
  hasAgentsMdInEntries,
  resolveAgentsMdStatus,
} from "./agentsMdStatus";
import type { MentionFileEntry } from "./projectFiles";

describe("agentsMdStatus", () => {
  it("returns idle without project", () => {
    expect(resolveAgentsMdStatus(undefined, false, [])).toBe("idle");
  });

  it("returns loading before index fetch completes", () => {
    expect(resolveAgentsMdStatus("p1", false, [])).toBe("loading");
  });

  it("detects AGENTS.md in file index", () => {
    const entries: MentionFileEntry[] = [
      { path: "AGENTS.md", isDir: false, modifiedMs: 1 },
    ];
    expect(hasAgentsMdInEntries(entries)).toBe(true);
    expect(resolveAgentsMdStatus("p1", true, entries)).toBe("loaded");
  });

  it("returns missing when index loaded without AGENTS.md", () => {
    expect(
      resolveAgentsMdStatus("p1", true, [{ path: "readme.md", isDir: false, modifiedMs: 1 }]),
    ).toBe("missing");
  });
});
