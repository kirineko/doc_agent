import { describe, expect, it } from "vitest";
import {
  composerWelcomeMessage,
  isComposerEmptyLayout,
  shouldCenterComposer,
} from "./composerLayout";

describe("composerLayout", () => {
  it("treats empty thread as empty layout", () => {
    expect(isComposerEmptyLayout(0, "", "", false)).toBe(true);
  });

  it("exits empty layout when messages exist or streaming", () => {
    expect(isComposerEmptyLayout(1, "", "", false)).toBe(false);
    expect(isComposerEmptyLayout(0, "think", "", false)).toBe(false);
    expect(isComposerEmptyLayout(0, "", "hello", false)).toBe(false);
    expect(isComposerEmptyLayout(0, "", "", true)).toBe(false);
  });

  it("centers composer without project or on empty layout", () => {
    expect(shouldCenterComposer(false, false)).toBe(true);
    expect(shouldCenterComposer(true, true)).toBe(true);
    expect(shouldCenterComposer(true, false)).toBe(false);
  });

  it("shows project onboarding copy without project", () => {
    expect(composerWelcomeMessage(false)).toContain("项目目录");
    expect(composerWelcomeMessage(true)).toMatch(/好/);
  });
});
