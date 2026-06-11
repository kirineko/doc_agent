import {
  AgentStreamState,
  applyAgentEvent,
  initialAgentStreamState,
  markAgentBusy,
  markAgentResuming,
  resetAgentStream,
} from "./agentEvents";
import type { AgentEvent } from "../types";

export type StreamAction =
  | { type: "event"; event: AgentEvent; sessionId?: string }
  | { type: "busy" }
  | { type: "busy_resume" }
  | { type: "reset" };

export function streamReducer(state: AgentStreamState, action: StreamAction): AgentStreamState {
  if (action.type === "reset") {
    return resetAgentStream();
  }
  if (action.type === "busy") {
    return markAgentBusy(state);
  }
  if (action.type === "busy_resume") {
    return markAgentResuming(state);
  }
  return applyAgentEvent(state, action.event, action.sessionId);
}

export { initialAgentStreamState };
