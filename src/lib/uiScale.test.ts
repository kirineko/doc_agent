import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  UI_SCALE_DEFAULT,
  UI_SCALE_STORAGE_KEY,
  applyUiScale,
  formatUiScalePercent,
  parseUiScale,
  readStoredUiScale,
  snapUiScale,
  stepUiScale,
  writeStoredUiScale,
} from "./uiScale";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: vi.fn(() => ({ label: "main" })),
}));

describe("uiScale", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("snaps to 0.2 steps within 100%-200%", () => {
    expect(snapUiScale(1)).toBe(1);
    expect(snapUiScale(1.14)).toBe(1.2);
    expect(snapUiScale(1.25)).toBe(1.2);
    expect(snapUiScale(1.34)).toBe(1.4);
    expect(snapUiScale(2)).toBe(2);
  });

  it("clamps out-of-range values", () => {
    expect(snapUiScale(0.9)).toBe(1);
    expect(snapUiScale(2.5)).toBe(2);
    expect(snapUiScale(Number.NaN)).toBe(UI_SCALE_DEFAULT);
  });

  it("parses invalid storage as default", () => {
    expect(parseUiScale(null)).toBe(1);
    expect(parseUiScale("not-a-number")).toBe(1);
    expect(parseUiScale("2.5")).toBe(2);
  });

  it("persists snapped scale in localStorage", () => {
    writeStoredUiScale(1.45);
    expect(localStorage.getItem(UI_SCALE_STORAGE_KEY)).toBe("1.4");
    expect(readStoredUiScale()).toBe(1.4);
  });

  it("steps by 0.2", () => {
    expect(stepUiScale(1.2, 0.2)).toBe(1.4);
    expect(stepUiScale(1, -0.2)).toBe(1);
    expect(stepUiScale(2, 0.2)).toBe(2);
  });

  it("formats percent label", () => {
    expect(formatUiScalePercent(1.4)).toBe("140%");
  });

  it("applyUiScale no-ops invoke outside Tauri", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const result = await applyUiScale(1.6);
    expect(result).toBe(1.6);
    expect(invoke).not.toHaveBeenCalled();
    expect(readStoredUiScale()).toBe(1.6);
  });
});
