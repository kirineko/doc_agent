import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { mergeProjectFilePaths, sameStringArrays } from "../lib/projectFiles";
import type { AgentEvent, ProjectFileList } from "../types";

const FILE_INDEX_DEBOUNCE_MS = 500;

export function useProjectFiles(projectId: string | undefined) {
  const [filePaths, setFilePaths] = useState<string[]>([]);
  const [fileRevision, setFileRevision] = useState(0);
  const projectIdRef = useRef(projectId);
  const filePathsRef = useRef<string[]>([]);
  const refreshSeqRef = useRef(0);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  projectIdRef.current = projectId;

  const applyFilePaths = useCallback((next: string[]) => {
    filePathsRef.current = next;
    setFilePaths(next);
  }, []);

  const fetchFilePaths = useCallback(
    async (id: string, bumpIfChanged: boolean) => {
      const seq = ++refreshSeqRef.current;
      try {
        const files = await invoke<ProjectFileList>("list_project_files_cmd", { projectId: id });
        if (seq !== refreshSeqRef.current || id !== projectIdRef.current) return;
        const next = files.entries.map((entry) => entry.path);
        const changed = !sameStringArrays(filePathsRef.current, next);
        applyFilePaths(next);
        if (bumpIfChanged && changed) {
          setFileRevision((value) => value + 1);
        }
      } catch (error) {
        console.error(error);
      }
    },
    [applyFilePaths],
  );

  const reset = useCallback(() => {
    refreshSeqRef.current += 1;
    if (debounceRef.current !== undefined) {
      clearTimeout(debounceRef.current);
      debounceRef.current = undefined;
    }
    applyFilePaths([]);
    setFileRevision(0);
  }, [applyFilePaths]);

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
      // turn 末兜底刷新：仅当文件清单实际变化才 bump revision，避免无谓的目录重读
      if (id) void fetchFilePaths(id, true);
    }, FILE_INDEX_DEBOUNCE_MS);
  }, [fetchFilePaths]);

  const onAgentEvent = useCallback(
    (event: AgentEvent) => {
      if (event.kind === "tool_result" && event.ok && event.changed_paths?.length) {
        // @ 索引按忽略规则增量 merge；explorer 一律 bump（OOXML 目录虽不进索引，但要在目录树可见）
        applyFilePaths(mergeProjectFilePaths(filePathsRef.current, event.changed_paths));
        setFileRevision((value) => value + 1);
      }
      if (event.kind === "turn_complete") {
        scheduleRefreshAll();
      }
    },
    [applyFilePaths, scheduleRefreshAll],
  );

  useEffect(() => {
    return () => {
      if (debounceRef.current !== undefined) {
        clearTimeout(debounceRef.current);
      }
    };
  }, []);

  return {
    filePaths,
    fileRevision,
    loadInitial,
    reset,
    onAgentEvent,
  };
}
