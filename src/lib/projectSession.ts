import type { Session } from "../types";

/** 按 updated_at 选取最近会话，与侧栏展示顺序无关。 */
export function mostRecentSessionId(sessions: Session[]): string | undefined {
  if (sessions.length === 0) return undefined;
  return sessions.reduce((best, session) =>
    session.updated_at > best.updated_at ? session : best,
  ).id;
}

export function shouldApplyProjectSelection(
  requestedProjectId: string | undefined,
  selectionTargetId: string | undefined,
): boolean {
  return requestedProjectId === selectionTargetId;
}
