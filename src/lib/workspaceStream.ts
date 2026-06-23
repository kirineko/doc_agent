import { AgentEvent } from "../types";
import {
  applyEventToSessionRuns,
  clearCompactionNotice,
  forceSessionIdle,
  initialSessionRunsState,
  markSessionCompacting,
  markSessionIdle,
  markSessionResuming,
  markSessionRunning,
  markSessionStopping,
  setCompactionNotice,
  type SessionRunsState,
} from "./sessionRunState";

export type StreamAction =
  | { type: "event"; event: AgentEvent }
  | { type: "busy"; sessionId: string }
  | { type: "busy_compact"; sessionId: string }
  | { type: "stopping"; sessionId: string }
  | { type: "busy_resume"; sessionId: string }
  | { type: "force_idle"; sessionId: string }
  | { type: "idle"; sessionId: string }
  | { type: "clear_compaction_notice"; sessionId: string }
  | { type: "compaction_notice"; sessionId: string; message: string };

export function sessionRunsReducer(
  state: SessionRunsState,
  action: StreamAction,
): SessionRunsState {
  switch (action.type) {
    case "event":
      return applyEventToSessionRuns(state, action.event);
    case "busy":
      return markSessionRunning(state, action.sessionId);
    case "busy_compact":
      return markSessionCompacting(state, action.sessionId);
    case "stopping":
      return markSessionStopping(state, action.sessionId);
    case "busy_resume":
      return markSessionResuming(state, action.sessionId);
    case "force_idle":
      return forceSessionIdle(state, action.sessionId);
    case "idle":
      return markSessionIdle(state, action.sessionId);
    case "clear_compaction_notice":
      return clearCompactionNotice(state, action.sessionId);
    case "compaction_notice":
      return setCompactionNotice(state, action.sessionId, action.message);
    default:
      return state;
  }
}

export { initialSessionRunsState };
