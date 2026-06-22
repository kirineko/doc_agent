import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  mergeProjectFileEntries,
  projectFileEntryFromApi,
  sameMentionFileEntries,
  type MentionFileEntry,
} from "../lib/projectFiles";
import type { AgentEvent, ProjectFileList } from "../types";

const FILE_INDEX_DEBOUNCE_MS = 500;

export function useProjectFiles(projectId: string | undefined) {
  const [fileEntries, setFileEntries] = useState<MentionFileEntry[]>([]);
  const [fileRevision, setFileRevision] = useState(0);
  const [filesLoaded, setFilesLoaded] = useState(false);
  const projectIdRef = useRef(projectId);
  const fileEntriesRef = useRef<MentionFileEntry[]>([]);
  const refreshSeqRef = useRef(0);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  projectIdRef.current = projectId;

  const applyFileEntries = useCallback((next: MentionFileEntry[]) => {
    fileEntriesRef.current = next;
    setFileEntries(next);
  }, []);

  const fetchFilePaths = useCallback(
    async (id: string, bumpIfChanged: boolean) => {
      const seq = ++refreshSeqRef.current;
      try {
        const files = await invoke<ProjectFileList>("list_project_files_cmd", { projectId: id });
        if (seq !== refreshSeqRef.current || id !== projectIdRef.current) return;
        const next = files.entries.map(projectFileEntryFromApi);
        const changed = !sameMentionFileEntries(fileEntriesRef.current, next);
        applyFileEntries(next);
        if (bumpIfChanged && changed) {
          setFileRevision((value) => value + 1);
        }
      } catch (error) {
        console.error(error);
      } finally {
        if (seq === refreshSeqRef.current && id === projectIdRef.current) {
          setFilesLoaded(true);
        }
      }
    },
    [applyFileEntries],
  );

  const reset = useCallback(() => {
    refreshSeqRef.current += 1;
    if (debounceRef.current !== undefined) {
      clearTimeout(debounceRef.current);
      debounceRef.current = undefined;
    }
    applyFileEntries([]);
    setFileRevision(0);
    setFilesLoaded(false);
  }, [applyFileEntries]);

  const loadInitial = useCallback(
    (id: string) => fetchFilePaths(id, false),
    [fetchFilePaths],
  );

  const scheduleRefreshAll = useCallback(() => {
    if (debounceRef.current !== undefined) {
      clearTimeout(debounceRef.current);
    }
    debounceRef.current = setTimeout(() => {
      debounceRef.current = undefined;
      const id = projectIdRef.current;
      if (id) void fetchFilePaths(id, true);
    }, FILE_INDEX_DEBOUNCE_MS);
  }, [fetchFilePaths]);

  const onAgentEvent = useCallback(
    (event: AgentEvent) => {
      if (event.kind === "tool_result" && event.ok && event.changed_paths?.length) {
        applyFileEntries(
          mergeProjectFileEntries(fileEntriesRef.current, event.changed_paths),
        );
        setFileRevision((value) => value + 1);
      }
      if (event.kind === "turn_complete") {
        scheduleRefreshAll();
      }
    },
    [applyFileEntries, scheduleRefreshAll],
  );

  useEffect(() => {
    return () => {
      if (debounceRef.current !== undefined) {
        clearTimeout(debounceRef.current);
      }
    };
  }, []);

  return {
    fileEntries,
    fileRevision,
    filesLoaded,
    loadInitial,
    reset,
    onAgentEvent,
    mergeImportedPaths: useCallback((paths: string[]) => {
      applyFileEntries(mergeProjectFileEntries(fileEntriesRef.current, paths));
      setFileRevision((value) => value + 1);
    }, [applyFileEntries]),
  };
}
