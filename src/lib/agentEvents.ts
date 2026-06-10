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

function clearStreamingBuffers(
  state: AgentStreamState,
  busy: boolean = state.busy,
): AgentStreamState {
  return {
    ...state,
    busy,
    streamingReasoning: "",
    streamingContent: "",
  };
}

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
    case "tool_call_stream": {
      const id = `streaming-${event.index}`;
      const entry: LiveToolCall = {
        id,
        name: event.name,
        args: undefined,
        status: "streaming",
        argsChars: event.args_chars,
      };
      const exists = state.liveTools.some((item) => item.id === id);
      return {
        ...state,
        liveTools: exists
          ? state.liveTools.map((item) => (item.id === id ? entry : item))
          : [...state.liveTools, entry],
      };
    }
    case "tool_call": {
      // 真实调用开始后，移除参数流式占位条目
      const liveTools = state.liveTools.filter(
        (item) => !item.id.startsWith("streaming-"),
      );
      const existing = liveTools.find((item) => item.id === event.id);
      if (existing) {
        return {
          ...state,
          liveTools: liveTools.map((item) =>
            item.id === event.id
              ? { ...item, status: event.status, args: event.args }
              : item,
          ),
        };
      }
      return {
        ...state,
        liveTools: [
          ...liveTools,
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
      return clearStreamingBuffers(state, false);
    case "assistant_step_done":
      return clearStreamingBuffers(state);
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
