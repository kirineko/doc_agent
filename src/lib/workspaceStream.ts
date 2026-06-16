import { AgentEvent } from "../types";
import {
  applyEventToSessionRuns,
  clearCompactionNotice,
  forceSessionIdle,
  initialSessionRunsState,
  markSessionResuming,
  markSessionRunning,
  markSessionStopping,
  type SessionRunsState,
} from "./sessionRunState";

export type StreamAction =
  | { type: "event"; event: AgentEvent }
  | { type: "busy"; sessionId: string }
  | { type: "stopping"; sessionId: string }
  | { type: "busy_resume"; sessionId: string }
  | { type: "force_idle"; sessionId: string }
  | { type: "clear_compaction_notice"; sessionId: string };

export function sessionRunsReducer(
  state: SessionRunsState,
  action: StreamAction,
): SessionRunsState {
  switch (action.type) {
    case "event":
      return applyEventToSessionRuns(state, action.event);
    case "busy":
      return markSessionRunning(state, action.sessionId);
    case "stopping":
      return markSessionStopping(state, action.sessionId);
    case "busy_resume":
      return markSessionResuming(state, action.sessionId);
    case "force_idle":
      return forceSessionIdle(state, action.sessionId);
    case "clear_compaction_notice":
      return clearCompactionNotice(state, action.sessionId);
    default:
      return state;
  }
}

export { initialSessionRunsState };
