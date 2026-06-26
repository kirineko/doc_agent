export const ACTIVE_WORKSPACE_STORAGE_KEY = "doc-agent-last-active-workspace";

export interface StoredActiveWorkspace {
  projectId: string;
  sessionId?: string;
}

export function parseStoredActiveWorkspace(value: unknown): StoredActiveWorkspace | undefined {
  if (!value || typeof value !== "object") return undefined;
  const record = value as Record<string, unknown>;
  if (typeof record.projectId !== "string" || !record.projectId.trim()) return undefined;
  const sessionId = record.sessionId;
  return {
    projectId: record.projectId,
    sessionId: typeof sessionId === "string" && sessionId.trim() ? sessionId : undefined,
  };
}

export function readStoredActiveWorkspace(
  storage: Storage = localStorage,
): StoredActiveWorkspace | undefined {
  try {
    const raw = storage.getItem(ACTIVE_WORKSPACE_STORAGE_KEY);
    if (!raw) return undefined;
    return parseStoredActiveWorkspace(JSON.parse(raw));
  } catch {
    return undefined;
  }
}

export function writeStoredActiveWorkspace(
  workspace: StoredActiveWorkspace,
  storage: Storage = localStorage,
): void {
  try {
    storage.setItem(ACTIVE_WORKSPACE_STORAGE_KEY, JSON.stringify(workspace));
  } catch {
    // ignore quota / private mode
  }
}

export function clearStoredActiveWorkspace(storage: Storage = localStorage): void {
  try {
    storage.removeItem(ACTIVE_WORKSPACE_STORAGE_KEY);
  } catch {
    // ignore quota / private mode
  }
}

export type WorkspaceRestorePlan =
  | { kind: "noop" }
  | { kind: "clear" }
  | { kind: "restore"; projectId: string; preferredSessionId?: string };

/** 根据项目列表与缓存决定冷启动恢复策略。 */
export function planWorkspaceRestore(
  projects: Array<{ id: string }>,
  stored?: StoredActiveWorkspace,
): WorkspaceRestorePlan {
  if (!stored?.projectId) return { kind: "noop" };
  if (projects.length === 0) return { kind: "noop" };
  if (!projects.some((project) => project.id === stored.projectId)) {
    return { kind: "clear" };
  }
  return {
    kind: "restore",
    projectId: stored.projectId,
    preferredSessionId: stored.sessionId,
  };
}
