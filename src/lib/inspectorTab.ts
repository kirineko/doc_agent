export type InspectorTab = "files" | "toolchain" | "artifacts";

/** @deprecated Legacy preference key; cleared on layout reset for migration only. */
export const INSPECTOR_TAB_STORAGE_KEY = "doc-agent-inspector-tab";
export const DEFAULT_INSPECTOR_TAB: InspectorTab = "files";

export function clearStoredInspectorTab(storage: Storage = localStorage): void {
  try {
    storage.removeItem(INSPECTOR_TAB_STORAGE_KEY);
  } catch {
    // ignore quota / private mode
  }
}

export function isActiveToolStatus(status: string): boolean {
  return status === "running" || status === "streaming";
}

/** Returns toolchain tab when auto-switch should fire; null otherwise. */
export function resolveInspectorAutoSwitch(
  tab: InspectorTab,
  userPinnedTab: InspectorTab | null,
  hasActiveTool: boolean,
  autoSwitchedThisTurn: boolean,
): InspectorTab | null {
  if (userPinnedTab !== null) return null;
  if (autoSwitchedThisTurn) return null;
  if (!hasActiveTool) return null;
  if (tab === "toolchain") return null;
  return "toolchain";
}
