import { describe, expect, it } from "vitest";
import { MODEL_OPTIONS } from "./types";

describe("MODEL_OPTIONS", () => {
  it("includes mock and production models", () => {
    const ids = MODEL_OPTIONS.map((item) => item.id);
    expect(ids).toContain("mock");
    expect(ids).toContain("deepseek-v4-flash");
    expect(ids).toContain("kimi-k2.6");
  });

  it("marks effort support correctly", () => {
    const deepseek = MODEL_OPTIONS.find((item) => item.id === "deepseek-v4-flash");
    const kimi = MODEL_OPTIONS.find((item) => item.id === "kimi-k2.6");
    expect(deepseek?.supportsEffort).toBe(true);
    expect(kimi?.supportsEffort).toBe(false);
  });
});
