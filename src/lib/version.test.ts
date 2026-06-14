import { describe, expect, it } from "vitest";
import { calVerForDate, formatCalVerDisplay, isValidCalVerVersion } from "./version";

describe("isValidCalVerVersion", () => {
  it("accepts valid CalVer examples", () => {
    expect(isValidCalVerVersion("2026.6.14")).toBe(true);
    expect(isValidCalVerVersion("2026.6.1")).toBe(true);
    expect(isValidCalVerVersion("2026.6.9")).toBe(true);
    expect(isValidCalVerVersion("2026.12.31")).toBe(true);
  });

  it("rejects leading zeros", () => {
    expect(isValidCalVerVersion("2026.06.14")).toBe(false);
    expect(isValidCalVerVersion("2026.6.09")).toBe(false);
    expect(isValidCalVerVersion("2026.06.09")).toBe(false);
  });

  it("rejects invalid ranges and shapes", () => {
    expect(isValidCalVerVersion("1.0.0")).toBe(false);
    expect(isValidCalVerVersion("2026.13.1")).toBe(false);
    expect(isValidCalVerVersion("2026.6.32")).toBe(false);
    expect(isValidCalVerVersion("v2026.6.14")).toBe(false);
  });
});

describe("formatCalVerDisplay", () => {
  it("formats single-digit month and day", () => {
    expect(formatCalVerDisplay("2026.6.9")).toBe("2026-06-09");
    expect(formatCalVerDisplay("2026.6.14")).toBe("2026-06-14");
  });
});

describe("calVerForDate", () => {
  it("builds YYYY.M.D without leading zeros", () => {
    expect(calVerForDate(new Date(2026, 5, 9))).toBe("2026.6.9");
  });
});
