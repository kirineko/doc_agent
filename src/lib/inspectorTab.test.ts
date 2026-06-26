import { describe, expect, it } from "vitest";
import {
  DEFAULT_INSPECTOR_TAB,
  INSPECTOR_TAB_STORAGE_KEY,
  isActiveToolStatus,
  clearStoredInspectorTab,
  resolveInspectorAutoSwitch,
} from "./inspectorTab";

describe("inspectorTab", () => {
  it("defaults to project files tab", () => {
    expect(DEFAULT_INSPECTOR_TAB).toBe("files");
  });

  it("clears legacy persisted tab preference", () => {
    const data: Record<string, string> = {
      [INSPECTOR_TAB_STORAGE_KEY]: "artifacts",
    };
    const storage = {
      getItem(key: string) {
        return data[key] ?? null;
      },
      setItem(key: string, value: string) {
        data[key] = value;
      },
      removeItem(key: string) {
        delete data[key];
      },
    } as Storage;

    clearStoredInspectorTab(storage);
    expect(storage.getItem(INSPECTOR_TAB_STORAGE_KEY)).toBeNull();
  });

  it("detects active tool statuses", () => {
    expect(isActiveToolStatus("running")).toBe(true);
    expect(isActiveToolStatus("streaming")).toBe(true);
    expect(isActiveToolStatus("done")).toBe(false);
  });

  it("auto-switches to toolchain when unpinned and tools active", () => {
    expect(resolveInspectorAutoSwitch("files", null, true, false)).toBe("toolchain");
    expect(resolveInspectorAutoSwitch("files", "files", true, false)).toBe(null);
    expect(resolveInspectorAutoSwitch("files", null, true, true)).toBe(null);
    expect(resolveInspectorAutoSwitch("toolchain", null, true, false)).toBe(null);
  });
});
