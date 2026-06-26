import { describe, expect, it } from "vitest";
import {
  ACTIVE_WORKSPACE_STORAGE_KEY,
  clearStoredActiveWorkspace,
  parseStoredActiveWorkspace,
  planWorkspaceRestore,
  readStoredActiveWorkspace,
  writeStoredActiveWorkspace,
} from "./activeWorkspace";

describe("activeWorkspace", () => {
  it("roundtrips project and session ids", () => {
    const storage = createMemoryStorage();
    writeStoredActiveWorkspace({ projectId: "p1", sessionId: "s1" }, storage);
    expect(readStoredActiveWorkspace(storage)).toEqual({
      projectId: "p1",
      sessionId: "s1",
    });
  });

  it("rejects invalid payloads", () => {
    expect(parseStoredActiveWorkspace(null)).toBeUndefined();
    expect(parseStoredActiveWorkspace({ projectId: "" })).toBeUndefined();
    expect(parseStoredActiveWorkspace({ projectId: "p1", sessionId: 1 })).toEqual({
      projectId: "p1",
      sessionId: undefined,
    });
  });

  it("clears stored workspace", () => {
    const storage = createMemoryStorage();
    writeStoredActiveWorkspace({ projectId: "p1", sessionId: "s1" }, storage);
    clearStoredActiveWorkspace(storage);
    expect(storage.getItem(ACTIVE_WORKSPACE_STORAGE_KEY)).toBeNull();
  });

  it("plans restore when project still exists", () => {
    expect(
      planWorkspaceRestore([{ id: "p1" }], { projectId: "p1", sessionId: "s1" }),
    ).toEqual({ kind: "restore", projectId: "p1", preferredSessionId: "s1" });
  });

  it("restores project without session when last workspace had no sessions", () => {
    expect(planWorkspaceRestore([{ id: "p1" }], { projectId: "p1" })).toEqual({
      kind: "restore",
      projectId: "p1",
      preferredSessionId: undefined,
    });
  });

  it("noops when there are no projects yet", () => {
    expect(planWorkspaceRestore([], { projectId: "p1", sessionId: "s1" })).toEqual({
      kind: "noop",
    });
  });

  it("clears stale workspace when project was removed", () => {
    expect(planWorkspaceRestore([{ id: "p2" }], { projectId: "p1", sessionId: "s1" })).toEqual({
      kind: "clear",
    });
  });
});

function createMemoryStorage(): Storage {
  const data = new Map<string, string>();
  return {
    get length() {
      return data.size;
    },
    clear() {
      data.clear();
    },
    getItem(key: string) {
      return data.get(key) ?? null;
    },
    key(index: number) {
      return [...data.keys()][index] ?? null;
    },
    removeItem(key: string) {
      data.delete(key);
    },
    setItem(key: string, value: string) {
      data.set(key, value);
    },
  } as Storage;
}
