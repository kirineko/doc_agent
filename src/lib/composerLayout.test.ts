import { describe, expect, it } from "vitest";
import {
  composerWelcomeMessage,
  greetingForHour,
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

  it("picks greeting by hour band", () => {
    expect(greetingForHour(0)).toContain("午夜");
    expect(greetingForHour(5)).toContain("午夜");
    expect(greetingForHour(6)).toContain("早上");
    expect(greetingForHour(11)).toContain("早上");
    expect(greetingForHour(12)).toContain("下午");
    expect(greetingForHour(17)).toContain("下午");
    expect(greetingForHour(18)).toContain("晚上");
    expect(greetingForHour(23)).toContain("晚上");
  });
});
