import { describe, expect, it } from "vitest";
import { hasAnyLlmKey } from "./credentials";

describe("hasAnyLlmKey", () => {
  it("returns false when no LLM provider has a key", () => {
    expect(hasAnyLlmKey({})).toBe(false);
    expect(hasAnyLlmKey({ tavily: true })).toBe(false);
  });

  it("returns true when any LLM provider has a key", () => {
    expect(hasAnyLlmKey({ deepseek: true })).toBe(true);
    expect(hasAnyLlmKey({ kimi: true, tavily: false })).toBe(true);
  });
});
