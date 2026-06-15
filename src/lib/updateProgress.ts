import type { DownloadEvent } from "@tauri-apps/plugin-updater";

export type UpdatePhase = "idle" | "downloading" | "installing";

export interface UpdateProgressState {
  phase: UpdatePhase;
  version?: string;
  downloadedBytes: number;
  totalBytes?: number;
}

const IDLE_STATE: UpdateProgressState = {
  phase: "idle",
  downloadedBytes: 0,
};

let state: UpdateProgressState = IDLE_STATE;
const listeners = new Set<() => void>();

function emit() {
  for (const listener of listeners) {
    listener();
  }
}

export function getUpdateProgressSnapshot(): UpdateProgressState {
  return state;
}

export function subscribeUpdateProgress(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function resetUpdateProgress(): void {
  state = IDLE_STATE;
  emit();
}

export function startUpdateDownload(version: string): void {
  state = {
    phase: "downloading",
    version,
    downloadedBytes: 0,
    totalBytes: undefined,
  };
  emit();
}

export function applyDownloadEvent(event: DownloadEvent): void {
  if (state.phase === "idle") return;

  if (event.event === "Started") {
    state = {
      ...state,
      totalBytes: event.data.contentLength,
    };
    emit();
    return;
  }

  if (event.event === "Progress") {
    state = {
      ...state,
      downloadedBytes: state.downloadedBytes + event.data.chunkLength,
    };
    emit();
    return;
  }

  if (event.event === "Finished") {
    state = {
      ...state,
      phase: "installing",
    };
    emit();
  }
}

export function computeUpdatePercent(progress: UpdateProgressState): number | undefined {
  if (progress.totalBytes === undefined || progress.totalBytes <= 0) return undefined;
  const ratio = progress.downloadedBytes / progress.totalBytes;
  return Math.min(100, Math.max(0, Math.round(ratio * 100)));
}

export function isUpdateInProgress(): boolean {
  return state.phase !== "idle";
}
