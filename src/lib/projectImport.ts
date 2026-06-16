import { formatMentionPath } from "./mention";

export const MAX_IMPORT_FILE_BYTES = 100 * 1024 * 1024;

export type ImportConflictAction = "overwrite" | "rename" | "cancel";

export interface ImportProjectFileResponse {
  path: string;
  renamed: boolean;
}

export function isImportFileExistsError(error: unknown): boolean {
  return String(error).includes("file already exists");
}

export function basenameFromPath(path: string): string {
  const normalized = path.replace(/\\/g, "/");
  const index = normalized.lastIndexOf("/");
  return index >= 0 ? normalized.slice(index + 1) : normalized;
}

/** 在当前光标处插入多个 @ 引用，末尾留空格 */
export function buildMentionInsert(
  text: string,
  cursor: number,
  paths: string[],
): { text: string; cursor: number } {
  if (paths.length === 0) {
    return { text, cursor };
  }
  const mentionText = paths.map((path) => `@${formatMentionPath(path)}`).join(" ") + " ";
  const next = `${text.slice(0, cursor)}${mentionText}${text.slice(cursor)}`;
  const nextCursor = cursor + mentionText.length;
  return { text: next, cursor: nextCursor };
}

export function mapConflictDialogResult(
  result: string | null,
  labels: { overwrite: string; rename: string },
): ImportConflictAction {
  if (result === labels.overwrite) return "overwrite";
  if (result === labels.rename) return "rename";
  return "cancel";
}

export const IMPORT_CONFLICT_LABELS = {
  overwrite: "覆盖",
  rename: "另存为",
  cancel: "取消",
} as const;

export function conflictStrategyForAction(
  action: ImportConflictAction,
): "overwrite" | "rename" | null {
  if (action === "overwrite") return "overwrite";
  if (action === "rename") return "rename";
  return null;
}
