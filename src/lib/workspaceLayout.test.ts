import { describe, expect, it, vi } from "vitest";
import {
  clearStoredWorkspaceLayouts,
  createValidatingLayoutStorage,
  DEFAULT_MAIN_LAYOUT,
  DEFAULT_RIGHT_LAYOUT,
  isValidStoredPanelLayout,
  MAIN_LAYOUT_GROUP_ID,
  MAIN_PANEL_IDS,
  onWorkspaceLayoutReset,
  parsePanelPercent,
  PANEL_COLLAPSED_SIZE,
  PANEL_MIN_SIZE,
  resetWorkspaceLayoutToDefaults,
  RIGHT_LAYOUT_GROUP_ID,
  RIGHT_PANEL_IDS,
  WORKSPACE_LAYOUT_RESET_EVENT,
  workspaceLayoutStorageKeys,
} from "./workspaceLayout";

describe("workspaceLayout", () => {
  it("exports stable layout group ids", () => {
    expect(MAIN_LAYOUT_GROUP_ID).toBe("doc-agent-layout-main");
    expect(RIGHT_LAYOUT_GROUP_ID).toBe("doc-agent-layout-right");
  });

  it("uses expected default main layout ratios", () => {
    expect(DEFAULT_MAIN_LAYOUT[MAIN_PANEL_IDS.sidebar]).toBe(20);
    expect(DEFAULT_MAIN_LAYOUT[MAIN_PANEL_IDS.chat]).toBe(60);
    expect(DEFAULT_MAIN_LAYOUT[MAIN_PANEL_IDS.right]).toBe(20);
    expect(
      DEFAULT_MAIN_LAYOUT[MAIN_PANEL_IDS.sidebar] +
        DEFAULT_MAIN_LAYOUT[MAIN_PANEL_IDS.chat] +
        DEFAULT_MAIN_LAYOUT[MAIN_PANEL_IDS.right],
    ).toBe(100);
  });

  it("uses 60/40 default right layout", () => {
    expect(DEFAULT_RIGHT_LAYOUT[RIGHT_PANEL_IDS.toolchain]).toBe(60);
    expect(DEFAULT_RIGHT_LAYOUT[RIGHT_PANEL_IDS.files]).toBe(40);
  });

  it("defines minimum panel sizes as percentages", () => {
    expect(PANEL_MIN_SIZE.chat).toBe("35%");
    expect(parsePanelPercent(PANEL_MIN_SIZE.chat)).toBeGreaterThan(
      parsePanelPercent(PANEL_MIN_SIZE.sidebar),
    );
    expect(PANEL_MIN_SIZE.toolchain).toBe("15%");
    expect(PANEL_MIN_SIZE.files).toBe("15%");
  });

  it("uses pixel collapsed size for section headers", () => {
    expect(PANEL_COLLAPSED_SIZE).toBe("32px");
  });

  it("builds react-resizable-panels storage keys", () => {
    expect(workspaceLayoutStorageKeys()).toEqual([
      "react-resizable-panels:doc-agent-layout-main:sidebar:chat:right",
      "react-resizable-panels:doc-agent-layout-right:toolchain:files",
    ]);
  });

  it("clears stored workspace layouts from localStorage", () => {
    localStorage.clear();
    for (const key of workspaceLayoutStorageKeys()) {
      localStorage.setItem(key, '{"sidebar":10}');
    }

    clearStoredWorkspaceLayouts();

    for (const key of workspaceLayoutStorageKeys()) {
      expect(localStorage.getItem(key)).toBeNull();
    }
  });

  it("dispatches reset event after clearing storage", () => {
    localStorage.clear();
    const listener = vi.fn();
    window.addEventListener(WORKSPACE_LAYOUT_RESET_EVENT, listener);

    resetWorkspaceLayoutToDefaults();

    expect(listener).toHaveBeenCalledTimes(1);
    window.removeEventListener(WORKSPACE_LAYOUT_RESET_EVENT, listener);
  });

  it("registers reset listener cleanup", () => {
    const listener = vi.fn();
    const cleanup = onWorkspaceLayoutReset(listener);
    resetWorkspaceLayoutToDefaults();
    expect(listener).toHaveBeenCalledTimes(1);
    cleanup();
    resetWorkspaceLayoutToDefaults();
    expect(listener).toHaveBeenCalledTimes(1);
  });

  it("accepts valid stored panel layout payloads", () => {
    const raw = JSON.stringify({
      [MAIN_PANEL_IDS.sidebar]: 20,
      [MAIN_PANEL_IDS.chat]: 60,
      [MAIN_PANEL_IDS.right]: 20,
    });
    expect(isValidStoredPanelLayout(raw, Object.values(MAIN_PANEL_IDS))).toBe(true);
  });

  it("rejects malformed or incomplete stored layouts", () => {
    expect(isValidStoredPanelLayout("not-json", Object.values(MAIN_PANEL_IDS))).toBe(false);
    expect(
      isValidStoredPanelLayout(
        JSON.stringify({ [MAIN_PANEL_IDS.sidebar]: 20 }),
        Object.values(MAIN_PANEL_IDS),
      ),
    ).toBe(false);
  });

  it("drops invalid layout payloads from storage on read", () => {
    localStorage.clear();
    const key = "react-resizable-panels:test";
    localStorage.setItem(key, "{bad");
    const storage = createValidatingLayoutStorage(Object.values(MAIN_PANEL_IDS));

    expect(storage.getItem(key)).toBeNull();
    expect(localStorage.getItem(key)).toBeNull();
  });
});
