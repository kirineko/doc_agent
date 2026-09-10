import { describe, expect, it } from "vitest";
import { findModel, MODEL_OPTIONS } from "./types";

describe("MODEL_OPTIONS", () => {
  it("includes production models with deepseek as default option", () => {
    const ids = MODEL_OPTIONS.map((item) => item.id);
    expect(ids).not.toContain("mock");
    expect(ids[0]).toBe("deepseek-flash");
    expect(ids.filter((id) => MODEL_OPTIONS.find((item) => item.id === id)?.selectable)).toEqual([
      "deepseek-flash",
      "mimo-v2.5",
      "mimo-v2.5-pro",
      "kimi-k3",
      "gemini-3.8-flash",
      "glm-5.3-flash",
    ]);
    expect(ids).toContain("kimi-k2.6");
  });

  it("marks effort support correctly", () => {
    const deepseek = MODEL_OPTIONS.find((item) => item.id === "deepseek-flash");
    const kimi = MODEL_OPTIONS.find((item) => item.id === "kimi-k2.6");
    expect(deepseek?.label).toBe("DeepSeek Flash");
    expect(deepseek?.api_model).toBe("deepseek-flash");
    expect(deepseek?.supports_vision).toBe(true);
    expect(deepseek?.supports_effort).toBe(true);
    expect(kimi?.supports_effort).toBe(false);
    expect(findModel(MODEL_OPTIONS, "deepseek-v4-flash")?.id).toBe("deepseek-flash");
  });
});
