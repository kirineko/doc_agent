import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { useProjectFiles } from "./useProjectFiles";

describe("useProjectFiles", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("loadInitial fetches project file entries", async () => {
    vi.mocked(invoke).mockResolvedValue({
      entries: [{ path: "notes.md", is_dir: false, modified_ms: 42 }],
    });

    const { result } = renderHook(() => useProjectFiles("project-1"));

    await act(async () => {
      await result.current.loadInitial("project-1");
    });

    expect(invoke).toHaveBeenCalledWith("list_project_files_cmd", {
      projectId: "project-1",
    });
    expect(result.current.fileEntries).toEqual([
      { path: "notes.md", isDir: false, modifiedMs: 42 },
    ]);
  });

  it("merges changed_paths on tool_result without full reload", async () => {
    vi.mocked(invoke).mockResolvedValue({ entries: [] });

    const { result } = renderHook(() => useProjectFiles("project-1"));

    act(() => {
      result.current.onAgentEvent({
        kind: "tool_result",
        session_id: "s1",
        turn_id: "t1",
        id: "call-1",
        name: "fs_write",
        ok: true,
        summary: "ok",
        duration_ms: 12,
        changed_paths: ["new-report.docx"],
      });
    });

    expect(result.current.fileEntries.map((entry) => entry.path)).toContain("new-report.docx");
    expect(result.current.fileRevision).toBe(1);
    expect(invoke).not.toHaveBeenCalled();
  });

  it("debounces full refresh on turn_complete", async () => {
    vi.useFakeTimers();
    vi.mocked(invoke).mockResolvedValue({
      entries: [{ path: "fresh.md", is_dir: false, modified_ms: 1 }],
    });

    const { result } = renderHook(() => useProjectFiles("project-1"));

    act(() => {
      result.current.onAgentEvent({
        kind: "turn_complete",
        session_id: "s1",
        turn_id: "t1",
      });
    });

    expect(invoke).not.toHaveBeenCalled();

    await act(async () => {
      await vi.advanceTimersByTimeAsync(500);
    });

    expect(invoke).toHaveBeenCalledWith("list_project_files_cmd", {
      projectId: "project-1",
    });
    expect(result.current.fileEntries[0]?.path).toBe("fresh.md");
    vi.useRealTimers();
  });

  it("reset clears entries and cancels pending debounce", async () => {
    vi.useFakeTimers();
    vi.mocked(invoke).mockResolvedValue({ entries: [] });

    const { result } = renderHook(() => useProjectFiles("project-1"));

    act(() => {
      result.current.onAgentEvent({
        kind: "turn_complete",
        session_id: "s1",
        turn_id: "t1",
      });
      result.current.reset();
    });

    await act(async () => {
      vi.advanceTimersByTime(500);
    });

    expect(result.current.fileEntries).toEqual([]);
    expect(invoke).not.toHaveBeenCalled();
    vi.useRealTimers();
  });
});
