import type { Session } from "../types";

/** 后端 list_sessions 已按 updated_at DESC 排序，首项即最近会话。 */
export function mostRecentSessionId(sessions: Session[]): string | undefined {
  return sessions[0]?.id;
}

export function shouldApplyProjectSelection(
  requestedProjectId: string | undefined,
  selectionTargetId: string | undefined,
): boolean {
  return requestedProjectId === selectionTargetId;
}
