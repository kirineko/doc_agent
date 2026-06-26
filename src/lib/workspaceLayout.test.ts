import { describe, expect, it, vi } from "vitest";
import {
  clearLegacyWorkspaceLayoutKeys,
  clearStoredWorkspaceLayouts,
  DEFAULT_MAIN_LAYOUT,
  isValidStoredPanelLayout,
  MAIN_LAYOUT_GROUP_ID,
  MAIN_PANEL_IDS,
  PANEL_MIN_SIZE,
  parsePanelPercent,
  resetWorkspaceLayoutToDefaults,
  WORKSPACE_LAYOUT_RESET_EVENT,
  workspaceLayoutStorageKeys,
} from "./workspaceLayout";

describe("workspaceLayout", () => {
  it("exports stable layout group ids", () => {
    expect(MAIN_LAYOUT_GROUP_ID).toBe("doc-agent-layout-main");
  });

  it("uses expected default main layout ratios", () => {
    expect(DEFAULT_MAIN_LAYOUT[MAIN_PANEL_IDS.sidebar]).toBe(20);
    expect(DEFAULT_MAIN_LAYOUT[MAIN_PANEL_IDS.chat]).toBe(60);
    expect(DEFAULT_MAIN_LAYOUT[MAIN_PANEL_IDS.right]).toBe(20);
  });

  it("defines minimum panel sizes as percentages", () => {
    expect(PANEL_MIN_SIZE.chat).toBe("35%");
    expect(parsePanelPercent(PANEL_MIN_SIZE.chat)).toBeGreaterThan(
      parsePanelPercent(PANEL_MIN_SIZE.sidebar),
    );
  });

  it("builds react-resizable-panels storage keys for main layout only", () => {
    expect(workspaceLayoutStorageKeys()).toEqual([
      "react-resizable-panels:doc-agent-layout-main:sidebar:chat:right",
    ]);
  });

  it("clears stored workspace layouts from localStorage", () => {
    localStorage.clear();
    for (const key of workspaceLayoutStorageKeys()) {
      localStorage.setItem(key, '{"sidebar":10}');
    }
    localStorage.setItem(
      "react-resizable-panels:doc-agent-layout-right:toolchain:files",
      '{"toolchain":50}',
    );

    clearStoredWorkspaceLayouts();

    for (const key of workspaceLayoutStorageKeys()) {
      expect(localStorage.getItem(key)).toBeNull();
    }
    expect(
      localStorage.getItem("react-resizable-panels:doc-agent-layout-right:toolchain:files"),
    ).toBeNull();
  });

  it("clears legacy right vertical layout key on bootstrap helper", () => {
    localStorage.setItem(
      "react-resizable-panels:doc-agent-layout-right:toolchain:files",
      '{"toolchain":50}',
    );

    clearLegacyWorkspaceLayoutKeys();

    expect(
      localStorage.getItem("react-resizable-panels:doc-agent-layout-right:toolchain:files"),
    ).toBeNull();
  });

  it("dispatches reset event after clearing storage", () => {
    localStorage.clear();
    const listener = vi.fn();
    window.addEventListener(WORKSPACE_LAYOUT_RESET_EVENT, listener);

    resetWorkspaceLayoutToDefaults();

    expect(listener).toHaveBeenCalledTimes(1);
    window.removeEventListener(WORKSPACE_LAYOUT_RESET_EVENT, listener);
  });

  it("accepts valid stored panel layout payloads", () => {
    const raw = JSON.stringify({
      [MAIN_PANEL_IDS.sidebar]: 20,
      [MAIN_PANEL_IDS.chat]: 60,
      [MAIN_PANEL_IDS.right]: 20,
    });
    expect(isValidStoredPanelLayout(raw, Object.values(MAIN_PANEL_IDS))).toBe(true);
  });
});
