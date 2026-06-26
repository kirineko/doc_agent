import type { Session } from "../types";

/** 按 updated_at 选取最近会话，与侧栏展示顺序无关。 */
export function mostRecentSessionId(sessions: Session[]): string | undefined {
  if (sessions.length === 0) return undefined;
  return sessions.reduce((best, session) =>
    session.updated_at > best.updated_at ? session : best,
  ).id;
}

/** 冷启动恢复时优先使用上次会话，无效则回退到最近更新会话。 */
export function resolveInitialSessionId(
  sessions: Session[],
  preferredSessionId?: string,
): string | undefined {
  if (
    preferredSessionId &&
    sessions.some((session) => session.id === preferredSessionId)
  ) {
    return preferredSessionId;
  }
  return mostRecentSessionId(sessions);
}

export function shouldApplyProjectSelection(
  requestedProjectId: string | undefined,
  selectionTargetId: string | undefined,
): boolean {
  return requestedProjectId === selectionTargetId;
}
