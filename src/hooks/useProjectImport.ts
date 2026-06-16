import { invoke } from "@tauri-apps/api/core";
import { message } from "@tauri-apps/plugin-dialog";
import { useCallback, useState } from "react";
import { blobToBase64 } from "../lib/attachments";
import {
  basenameFromPath,
  buildMentionInsert,
  conflictStrategyForAction,
  IMPORT_CONFLICT_LABELS,
  type ImportProjectFileResponse,
  isImportFileExistsError,
  mapConflictDialogResult,
  MAX_IMPORT_FILE_BYTES,
} from "../lib/projectImport";
import type { SendBlocker } from "../lib/sendReadiness";

interface UseProjectImportOptions {
  projectId: string | undefined;
  input: string;
  cursor: number;
  setInput: (value: string) => void;
  setCursor: (value: number) => void;
  onFocusInput?: (start: number, end?: number) => void;
  mergeImportedPaths: (paths: string[]) => void;
  showSendBlocker: (blocker: SendBlocker) => void;
  ensureActiveSession?: () => Promise<string | null>;
  disabled: boolean;
}

async function importOneFile(
  projectId: string,
  filename: string,
  dataBase64: string,
  onConflict: "fail_if_exists" | "overwrite" | "rename",
): Promise<ImportProjectFileResponse> {
  return invoke<ImportProjectFileResponse>("import_project_file_cmd", {
    req: {
      project_id: projectId,
      filename,
      data_base64: dataBase64,
      on_conflict: onConflict,
    },
  });
}

async function resolveConflict(filename: string): Promise<"overwrite" | "rename" | "cancel"> {
  const result = await message(`「${filename}」已存在，如何处理？`, {
    title: "文件已存在",
    kind: "warning",
    buttons: {
      yes: IMPORT_CONFLICT_LABELS.overwrite,
      no: IMPORT_CONFLICT_LABELS.rename,
      cancel: IMPORT_CONFLICT_LABELS.cancel,
    },
  });
  return mapConflictDialogResult(result, IMPORT_CONFLICT_LABELS);
}

export function useProjectImport({
  projectId,
  input,
  cursor,
  setInput,
  setCursor,
  onFocusInput,
  mergeImportedPaths,
  showSendBlocker,
  ensureActiveSession,
  disabled,
}: UseProjectImportOptions) {
  const [importing, setImporting] = useState(false);

  const importProjectFiles = useCallback(
    async (files: File[]) => {
      if (disabled || importing) return;
      if (!projectId) {
        showSendBlocker({ kind: "no_project" });
        return;
      }
      if (files.length === 0) return;

      setImporting(true);
      try {
        if (ensureActiveSession) {
          await ensureActiveSession();
        }

        const importedPaths: string[] = [];

        for (const file of files) {
          const filename = basenameFromPath(file.name);
          if (file.size > MAX_IMPORT_FILE_BYTES) {
            await message(
              `「${filename}」超过 ${MAX_IMPORT_FILE_BYTES / 1024 / 1024}MB 限制，已跳过。`,
              { title: "文件过大", kind: "warning" },
            );
            continue;
          }

          let dataBase64: string;
          try {
            dataBase64 = await blobToBase64(file);
          } catch (error) {
            console.error(error);
            continue;
          }

          let onConflict: "fail_if_exists" | "overwrite" | "rename" = "fail_if_exists";
          let imported = false;

          while (!imported) {
            try {
              const response = await importOneFile(projectId, filename, dataBase64, onConflict);
              importedPaths.push(response.path);
              imported = true;
            } catch (error) {
              if (!isImportFileExistsError(error) || onConflict !== "fail_if_exists") {
                console.error(error);
                await message(String(error), { title: "导入失败", kind: "error" });
                break;
              }
              const action = await resolveConflict(filename);
              const strategy = conflictStrategyForAction(action);
              if (!strategy) break;
              onConflict = strategy;
            }
          }
        }

        if (importedPaths.length === 0) return;

        mergeImportedPaths(importedPaths);
        const inserted = buildMentionInsert(input, cursor, importedPaths);
        setInput(inserted.text);
        setCursor(inserted.cursor);
        onFocusInput?.(inserted.cursor);
      } finally {
        setImporting(false);
      }
    },
    [
      cursor,
      disabled,
      importing,
      input,
      mergeImportedPaths,
      onFocusInput,
      projectId,
      setCursor,
      setInput,
      showSendBlocker,
      ensureActiveSession,
    ],
  );

  return { importProjectFiles, importing };
}
