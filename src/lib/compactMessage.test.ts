import { describe, expect, it } from "vitest";
import { isCompactMessage } from "./compactMessage";

describe("isCompactMessage", () => {
  it("detects exact /compact", () => {
    expect(isCompactMessage("/compact")).toBe(true);
    expect(isCompactMessage(" /compact")).toBe(true);
    expect(isCompactMessage("  /compact  ")).toBe(true);
  });

  it("rejects tails and similar commands", () => {
    expect(isCompactMessage("/compact now")).toBe(false);
    expect(isCompactMessage("/compactify")).toBe(false);
    expect(isCompactMessage("/init")).toBe(false);
  });
});
