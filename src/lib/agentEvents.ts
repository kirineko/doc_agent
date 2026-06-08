import { LiveToolCall } from "../components/ToolChainPanel";
import { AgentEvent } from "../types";

export interface AgentStreamState {
  streamingReasoning: string;
  streamingContent: string;
  liveTools: LiveToolCall[];
  busy: boolean;
}

export const initialAgentStreamState: AgentStreamState = {
  streamingReasoning: "",
  streamingContent: "",
  liveTools: [],
  busy: false,
};

export function applyAgentEvent(
  state: AgentStreamState,
  event: AgentEvent,
  activeSessionId: string | undefined,
): AgentStreamState {
  if (event.session_id !== activeSessionId) {
    return state;
  }

  switch (event.kind) {
    case "reasoning_token":
      return {
        ...state,
        streamingReasoning: state.streamingReasoning + event.delta,
      };
    case "content_token":
      return {
        ...state,
        streamingContent: state.streamingContent + event.delta,
      };
    case "tool_call": {
      const existing = state.liveTools.find((item) => item.id === event.id);
      if (existing) {
        return {
          ...state,
          liveTools: state.liveTools.map((item) =>
            item.id === event.id
              ? { ...item, status: event.status, args: event.args }
              : item,
          ),
        };
      }
      return {
        ...state,
        liveTools: [
          ...state.liveTools,
          {
            id: event.id,
            name: event.name,
            args: event.args,
            status: event.status,
          },
        ],
      };
    }
    case "tool_result":
      return {
        ...state,
        liveTools: state.liveTools.map((item) =>
          item.id === event.id
            ? {
                ...item,
                status: event.ok ? "done" : "error",
              }
            : item,
        ),
      };
    case "turn_complete":
      return {
        ...state,
        busy: false,
        streamingReasoning: "",
        streamingContent: "",
      };
    case "error":
      return {
        ...state,
        busy: false,
        streamingContent: `${state.streamingContent}\n\n> ${event.message}`,
      };
    default:
      return state;
  }
}

export function markAgentBusy(state: AgentStreamState): AgentStreamState {
  return {
    ...state,
    busy: true,
    liveTools: [],
    streamingReasoning: "",
    streamingContent: "",
  };
}
