import { describe, expect, it } from "vitest";
import { isSessionModelLocked } from "./sessionConfig";

describe("isSessionModelLocked", () => {
  it("unlocks empty session", () => {
    expect(isSessionModelLocked(0)).toBe(false);
  });

  it("locks after first chat message", () => {
    expect(isSessionModelLocked(1)).toBe(true);
  });
});
