import { LiveToolCall } from "../components/ToolChainPanel";
import {
  compactionInProgressNotice,
  isCompactionInProgressNotice,
} from "./compactionNotice";
import { toolLabel } from "./toolLabels";
import { AgentEvent } from "../types";

/** 本轮由 Agent 产生或修改的交付物（已去重）。 */
export interface TurnArtifact {
  path: string;
  sourceToolCallId: string;
  sourceToolLabel: string;
}

export interface AgentStreamState {
  streamingReasoning: string;
  streamingContent: string;
  liveTools: LiveToolCall[];
  turnArtifacts: TurnArtifact[];
  busy: boolean;
  compactionNotice?: string | null;
}

export const initialAgentStreamState: AgentStreamState = {
  streamingReasoning: "",
  streamingContent: "",
  liveTools: [],
  turnArtifacts: [],
  busy: false,
  compactionNotice: null,
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

function dropStreamingPlaceholders(liveTools: LiveToolCall[]): LiveToolCall[] {
  return liveTools.filter((item) => !item.id.startsWith("streaming-"));
}

export function applyAgentEvent(
  state: AgentStreamState,
  event: AgentEvent,
  activeSessionId?: string,
): AgentStreamState {
  if (activeSessionId !== undefined && event.session_id !== activeSessionId) {
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
      const streamIndex = event.index ?? 0;
      const streamId = `streaming-${streamIndex}`;
      const streamPos = state.liveTools.findIndex((item) => item.id === streamId);
      if (streamPos >= 0) {
        const liveTools = [...state.liveTools];
        liveTools[streamPos] = {
          id: event.id,
          name: event.name,
          args: event.args,
          status: event.status,
        };
        return { ...state, liveTools };
      }
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
    case "tool_result": {
      // tool_result 事件不带 name，从 liveTools 查出来源工具标签
      const source = state.liveTools.find((item) => item.id === event.id);
      const liveTools = state.liveTools.map((item) =>
        item.id === event.id
          ? {
              ...item,
              status: event.ok ? "done" : "error",
              summary: event.summary,
              changed_paths: event.changed_paths,
            }
          : item,
      );
      // 把成功工具的 changed_paths 累积成本轮产物（按 path 去重，保留首个来源工具）
      if (event.ok && event.changed_paths?.length) {
        const sourceLabel = toolLabel(source?.name ?? "");
        const existing = new Set(state.turnArtifacts.map((a) => a.path));
        const added: TurnArtifact[] = [];
        for (const path of event.changed_paths) {
          if (path && !existing.has(path)) {
            existing.add(path);
            added.push({
              path,
              sourceToolCallId: event.id,
              sourceToolLabel: sourceLabel,
            });
          }
        }
        if (added.length > 0) {
          return {
            ...state,
            liveTools,
            turnArtifacts: [...state.turnArtifacts, ...added],
          };
        }
      }
      return { ...state, liveTools };
    }
    case "turn_complete":
      return {
        ...clearStreamingBuffers(state, false),
        liveTools: dropStreamingPlaceholders(state.liveTools),
      };
    case "turn_cancelled":
      return {
        ...clearStreamingBuffers(state, false),
        liveTools: dropStreamingPlaceholders(state.liveTools),
        compactionNotice: isCompactionInProgressNotice(state.compactionNotice)
          ? null
          : state.compactionNotice,
      };
    case "turn_awaiting_user":
      return {
        ...clearStreamingBuffers(state, false),
        liveTools: dropStreamingPlaceholders(state.liveTools),
      };
    case "assistant_step_done":
      return clearStreamingBuffers(state);
    case "context_usage":
      return state;
    case "compaction_started":
      return {
        ...state,
        compactionNotice: compactionInProgressNotice(event.trigger),
      };
    case "context_compacted":
      return {
        ...state,
        compactionNotice:
          event.trigger === "manual"
            ? "已手动压缩对话历史"
            : "已自动压缩较早的对话历史以节省上下文",
      };
    case "error":
      return {
        ...state,
        busy: false,
        compactionNotice: isCompactionInProgressNotice(state.compactionNotice)
          ? null
          : state.compactionNotice,
        streamingContent: `${state.streamingContent}\n\n> ${event.message}`,
      };
    default:
      return state;
  }
}

/** 新用户消息开始新 turn：清空工具链、产物列表与流式缓冲 */
export function markAgentBusy(state: AgentStreamState): AgentStreamState {
  return {
    ...state,
    busy: true,
    liveTools: [],
    turnArtifacts: [],
    streamingReasoning: "",
    streamingContent: "",
  };
}

/** 澄清提交后 resume 同一 turn：保留已有工具链，仅清流式缓冲 */
export function markAgentResuming(state: AgentStreamState): AgentStreamState {
  return {
    ...state,
    busy: true,
    streamingReasoning: "",
    streamingContent: "",
  };
}

export function resetAgentStream(): AgentStreamState {
  return initialAgentStreamState;
}

export function isTerminalRunEvent(kind: AgentEvent["kind"]): boolean {
  return (
    kind === "turn_complete" ||
    kind === "turn_cancelled" ||
    kind === "turn_awaiting_user" ||
    kind === "error"
  );
}
