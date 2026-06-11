import { afterEach, beforeEach, describe, expect, it } from "vitest";
import {
  THEME_STORAGE_KEY,
  applyTheme,
  parseTheme,
  readStoredTheme,
  writeStoredTheme,
} from "./theme";

describe("theme", () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.removeAttribute("data-theme");
  });

  afterEach(() => {
    localStorage.clear();
    document.documentElement.removeAttribute("data-theme");
  });

  it("parseTheme defaults invalid values to light", () => {
    expect(parseTheme(null)).toBe("light");
    expect(parseTheme(undefined)).toBe("light");
    expect(parseTheme("system")).toBe("light");
    expect(parseTheme("light")).toBe("light");
    expect(parseTheme("dark")).toBe("dark");
  });

  it("persists theme in localStorage", () => {
    writeStoredTheme("light");
    expect(localStorage.getItem(THEME_STORAGE_KEY)).toBe("light");
    expect(readStoredTheme()).toBe("light");
  });

  it("readStoredTheme falls back to light for invalid storage", () => {
    localStorage.setItem(THEME_STORAGE_KEY, "invalid");
    expect(readStoredTheme()).toBe("light");
  });

  it("applyTheme sets data-theme on documentElement", () => {
    applyTheme("light");
    expect(document.documentElement.dataset.theme).toBe("light");
    applyTheme("dark");
    expect(document.documentElement.dataset.theme).toBe("dark");
  });
});
