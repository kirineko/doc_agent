import { clearStoredInspectorTab } from "./inspectorTab";

export const MAIN_LAYOUT_GROUP_ID = "doc-agent-layout-main";

export const MAIN_PANEL_IDS = {
  sidebar: "sidebar",
  chat: "chat",
  right: "right",
} as const;

export const DEFAULT_MAIN_LAYOUT: Record<string, number> = {
  [MAIN_PANEL_IDS.sidebar]: 20,
  [MAIN_PANEL_IDS.chat]: 60,
  [MAIN_PANEL_IDS.right]: 20,
};

/** Panel minSize/defaultSize props: numeric values are pixels in v4; use explicit percentages. */
export const PANEL_MIN_SIZE = {
  sidebar: "12%",
  chat: "35%",
  right: "12%",
} as const;

export const PANEL_DEFAULT_SIZE = {
  sidebar: "20%",
  chat: "60%",
  right: "20%",
} as const;

const LAYOUT_STORAGE_PREFIX = "react-resizable-panels";

export const WORKSPACE_LAYOUT_RESET_EVENT = "doc-agent-workspace-layout-reset";

const LEGACY_RIGHT_LAYOUT_STORAGE_KEY =
  "react-resizable-panels:doc-agent-layout-right:toolchain:files";

function layoutStorageKey(groupId: string, panelIds: readonly string[]): string {
  return `${LAYOUT_STORAGE_PREFIX}:${[groupId, ...panelIds].join(":")}`;
}

export function workspaceLayoutStorageKeys(): string[] {
  return [layoutStorageKey(MAIN_LAYOUT_GROUP_ID, Object.values(MAIN_PANEL_IDS))];
}

export function clearLegacyWorkspaceLayoutKeys(storage: Storage = localStorage): void {
  try {
    storage.removeItem(LEGACY_RIGHT_LAYOUT_STORAGE_KEY);
  } catch {
    // ignore quota / private mode
  }
}

export function clearStoredWorkspaceLayouts(): void {
  for (const key of workspaceLayoutStorageKeys()) {
    try {
      localStorage.removeItem(key);
    } catch {
      // ignore quota / private mode
    }
  }
  clearLegacyWorkspaceLayoutKeys();
  clearStoredInspectorTab();
}

export function resetWorkspaceLayoutToDefaults(): void {
  clearStoredWorkspaceLayouts();
  window.dispatchEvent(new CustomEvent(WORKSPACE_LAYOUT_RESET_EVENT));
}

export function onWorkspaceLayoutReset(listener: () => void): () => void {
  window.addEventListener(WORKSPACE_LAYOUT_RESET_EVENT, listener);
  return () => window.removeEventListener(WORKSPACE_LAYOUT_RESET_EVENT, listener);
}

type LayoutStorage = Pick<Storage, "getItem" | "setItem" | "removeItem">;

export function parsePanelPercent(value: string): number {
  return Number.parseFloat(value.replace("%", ""));
}

export function isValidStoredPanelLayout(raw: string, panelIds: readonly string[]): boolean {
  try {
    const parsed: unknown = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) return false;
    const record = parsed as Record<string, unknown>;
    for (const id of panelIds) {
      const size = record[id];
      if (typeof size !== "number" || !Number.isFinite(size) || size < 0) return false;
    }
    return true;
  } catch {
    return false;
  }
}

export function createValidatingLayoutStorage(
  panelIds: readonly string[],
  underlying: Storage = localStorage,
): LayoutStorage {
  return {
    getItem(key: string): string | null {
      try {
        const raw = underlying.getItem(key);
        if (raw === null) return null;
        if (!isValidStoredPanelLayout(raw, panelIds)) {
          underlying.removeItem(key);
          return null;
        }
        return raw;
      } catch {
        try {
          underlying.removeItem(key);
        } catch {
          // ignore quota / private mode
        }
        return null;
      }
    },
    setItem(key: string, value: string): void {
      underlying.setItem(key, value);
    },
    removeItem(key: string): void {
      underlying.removeItem(key);
    },
  };
}

export const mainLayoutStorage = createValidatingLayoutStorage(Object.values(MAIN_PANEL_IDS));
