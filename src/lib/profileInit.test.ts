import { describe, expect, it } from "vitest";
import { isProfileInitMessage } from "./profileInit";

describe("isProfileInitMessage", () => {
  it("detects /init and tail", () => {
    expect(isProfileInitMessage("/init")).toBe(true);
    expect(isProfileInitMessage("/init 固化PPT")).toBe(true);
    expect(isProfileInitMessage(" /init")).toBe(true);
    expect(isProfileInitMessage("  /init 固化PPT")).toBe(true);
  });

  it("rejects non-init commands", () => {
    expect(isProfileInitMessage("/initialize")).toBe(false);
  });
});
