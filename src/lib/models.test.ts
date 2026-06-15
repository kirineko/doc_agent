import { describe, expect, it } from "vitest";
import { modelSupportsVision, providerForModel } from "./models";

describe("models helpers", () => {
  it("falls back to MODEL_OPTIONS for vision before catalog loads", () => {
    expect(modelSupportsVision([], "kimi-k2.6")).toBe(true);
    expect(modelSupportsVision([], "deepseek-v4-flash")).toBe(false);
  });

  it("prefers catalog entry when available", () => {
    expect(
      modelSupportsVision(
        [{
          id: "kimi-k2.6",
          label: "Kimi",
          provider: "kimi",
          api_model: "kimi-k2.6",
          supports_vision: false,
          supports_effort: false,
          max_context: 100000,
        }],
        "kimi-k2.6",
      ),
    ).toBe(false);
  });

  it("falls back provider lookup before catalog loads", () => {
    expect(providerForModel([], "mimo-v2.5")).toBe("mimo");
  });
});
