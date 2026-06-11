import {
  AgentStreamState,
  applyAgentEvent,
  initialAgentStreamState,
  markAgentBusy,
} from "./agentEvents";
import type { AgentEvent } from "../types";

export type StreamAction =
  | { type: "event"; event: AgentEvent; sessionId?: string }
  | { type: "busy" };

export function streamReducer(state: AgentStreamState, action: StreamAction): AgentStreamState {
  if (action.type === "busy") {
    return markAgentBusy(state);
  }
  return applyAgentEvent(state, action.event, action.sessionId);
}

export { initialAgentStreamState };
