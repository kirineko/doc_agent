import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";

export const UI_SCALE_STORAGE_KEY = "doc-agent-ui-scale";
export const UI_SCALE_STEP = 0.2;
export const UI_SCALE_MIN = 1;
export const UI_SCALE_MAX = 2;
export const UI_SCALE_DEFAULT = 1;

export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function snapUiScale(value: number): number {
  if (!Number.isFinite(value)) return UI_SCALE_DEFAULT;
  const snapped = Math.round(value / UI_SCALE_STEP) * UI_SCALE_STEP;
  const clamped = Math.min(UI_SCALE_MAX, Math.max(UI_SCALE_MIN, snapped));
  return Math.round(clamped * 10) / 10;
}

export function parseUiScale(value: string | null | undefined): number {
  if (value == null || value.trim() === "") return UI_SCALE_DEFAULT;
  const parsed = Number(value);
  return snapUiScale(parsed);
}

export function readStoredUiScale(storage: Storage = localStorage): number {
  try {
    return parseUiScale(storage.getItem(UI_SCALE_STORAGE_KEY));
  } catch {
    return UI_SCALE_DEFAULT;
  }
}

export function writeStoredUiScale(scale: number, storage: Storage = localStorage): void {
  try {
    storage.setItem(UI_SCALE_STORAGE_KEY, String(snapUiScale(scale)));
  } catch {
    // ignore quota / private mode
  }
}

export function formatUiScalePercent(scale: number): string {
  return `${Math.round(snapUiScale(scale) * 100)}%`;
}

export function stepUiScale(current: number, delta: number): number {
  return snapUiScale(current + delta);
}

export async function applyWebviewZoom(scale: number): Promise<void> {
  const snapped = snapUiScale(scale);
  if (!isTauriRuntime()) return;

  const webview = getCurrentWebview();
  await invoke("plugin:webview|set_webview_zoom", {
    label: webview.label,
    value: snapped,
  });
}

export async function applyUiScale(
  scale: number,
  storage: Storage = localStorage,
): Promise<number> {
  const snapped = snapUiScale(scale);
  await applyWebviewZoom(snapped);
  writeStoredUiScale(snapped, storage);
  return snapped;
}
