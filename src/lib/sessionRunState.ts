import { AgentEvent } from "../types";
import {
  applyAgentEvent,
  initialAgentStreamState,
  markAgentBusy,
  markAgentResuming,
  type AgentStreamState,
} from "./agentEvents";

export type SessionRunStatus = "idle" | "running" | "stopping";

export const MAX_PARALLEL_TURNS = 3;
export const PARALLEL_LIMIT_MESSAGE = "当前已有 3 个任务正在执行，请稍后重试。";
export const STOPPING_TIMEOUT_MS = 35_000;
export const STOPPING_TIMEOUT_SECONDS = Math.round(STOPPING_TIMEOUT_MS / 1000);

export interface SessionRunState extends AgentStreamState {
  status: SessionRunStatus;
}

export interface SessionRunsState {
  bySession: Record<string, SessionRunState>;
}

export const initialSessionRunsState: SessionRunsState = {
  bySession: {},
};

function emptyRun(): SessionRunState {
  return {
    ...initialAgentStreamState,
    status: "idle",
  };
}

function getOrCreate(state: SessionRunsState, sessionId: string): SessionRunState {
  return state.bySession[sessionId] ?? emptyRun();
}

export function sessionRunStatus(
  state: SessionRunsState,
  sessionId: string | undefined,
): SessionRunStatus {
  if (!sessionId) return "idle";
  return state.bySession[sessionId]?.status ?? "idle";
}

export function countActiveSessionRuns(state: SessionRunsState): number {
  return Object.values(state.bySession).filter(
    (run) => run.status === "running" || run.status === "stopping",
  ).length;
}

export function isParallelAtCapacity(state: SessionRunsState): boolean {
  return countActiveSessionRuns(state) >= MAX_PARALLEL_TURNS;
}

export function deriveActiveStream(
  state: SessionRunsState,
  activeSessionId: string | undefined,
): AgentStreamState {
  if (!activeSessionId) {
    return initialAgentStreamState;
  }
  const run = state.bySession[activeSessionId];
  if (!run) {
    return initialAgentStreamState;
  }
  return {
    streamingReasoning: run.streamingReasoning,
    streamingContent: run.streamingContent,
    liveTools: run.liveTools,
    turnArtifacts: run.turnArtifacts,
    busy: run.status === "running" || run.status === "stopping",
    compactionNotice: run.compactionNotice ?? null,
  };
}

export function applyEventToSessionRuns(
  state: SessionRunsState,
  event: AgentEvent,
): SessionRunsState {
  const sessionId = event.session_id;
  const current = getOrCreate(state, sessionId);
  const nextStream = applyAgentEvent(current, event);
  let status = current.status;

  switch (event.kind) {
    case "turn_complete":
    case "turn_cancelled":
    case "turn_awaiting_user":
      status = "idle";
      break;
    case "error":
      status = "idle";
      break;
    case "context_compacted":
      if (event.trigger === "manual") {
        status = "idle";
      } else if (status === "idle" && startsRunning(event)) {
        status = "running";
      }
      break;
    default:
      if (status === "idle" && (nextStream.busy || startsRunning(event))) {
        status = "running";
      }
      break;
  }

  return {
    bySession: {
      ...state.bySession,
      [sessionId]: {
        ...nextStream,
        status,
        ...(event.kind === "context_compacted" && event.trigger === "manual"
          ? { busy: false }
          : {}),
      },
    },
  };
}

function startsRunning(event: AgentEvent): boolean {
  return (
    event.kind === "reasoning_token" ||
    event.kind === "content_token" ||
    event.kind === "tool_call_stream" ||
    event.kind === "tool_call" ||
    event.kind === "assistant_step_done" ||
    // Auto compaction fires mid-turn (already running); manual /compact runs
    // outside any turn and MUST NOT flip an idle session to running.
    (event.kind === "context_compacted" && event.trigger !== "manual")
  );
}

export function sessionRunStatusMap(
  state: SessionRunsState,
): Record<string, SessionRunStatus> {
  const out: Record<string, SessionRunStatus> = {};
  for (const [sessionId, run] of Object.entries(state.bySession)) {
    out[sessionId] = run.status;
  }
  return out;
}

export function markSessionRunning(
  state: SessionRunsState,
  sessionId: string,
): SessionRunsState {
  const current = getOrCreate(state, sessionId);
  return {
    bySession: {
      ...state.bySession,
      [sessionId]: {
        ...markAgentBusy(current),
        status: "running",
        compactionNotice: null,
      },
    },
  };
}

/**
 * 手动 /compact 不是新 turn：标记忙碌以显示「压缩中」，但 MUST NOT 清空
 * turnArtifacts（产物仅在用户发送新消息时清空，见 workspace-ui spec）。
 */
export function markSessionCompacting(
  state: SessionRunsState,
  sessionId: string,
): SessionRunsState {
  const current = getOrCreate(state, sessionId);
  return {
    bySession: {
      ...state.bySession,
      [sessionId]: {
        ...current,
        status: "running",
        busy: true,
      },
    },
  };
}

export function markSessionStopping(
  state: SessionRunsState,
  sessionId: string,
): SessionRunsState {
  const current = getOrCreate(state, sessionId);
  return {
    bySession: {
      ...state.bySession,
      [sessionId]: {
        ...current,
        status: "stopping",
        busy: true,
      },
    },
  };
}

export function markSessionResuming(
  state: SessionRunsState,
  sessionId: string,
): SessionRunsState {
  const current = getOrCreate(state, sessionId);
  return {
    bySession: {
      ...state.bySession,
      [sessionId]: {
        ...markAgentResuming(current),
        status: "running",
      },
    },
  };
}

export function forceSessionIdle(
  state: SessionRunsState,
  sessionId: string,
): SessionRunsState {
  return {
    bySession: {
      ...state.bySession,
      [sessionId]: emptyRun(),
    },
  };
}

/** End a frontend-only busy spell without clearing compaction notice. */
export function markSessionIdle(
  state: SessionRunsState,
  sessionId: string,
): SessionRunsState {
  const current = getOrCreate(state, sessionId);
  return {
    bySession: {
      ...state.bySession,
      [sessionId]: {
        ...current,
        status: "idle",
        busy: false,
        streamingReasoning: "",
        streamingContent: "",
        liveTools: [],
      },
    },
  };
}

export function clearCompactionNotice(
  state: SessionRunsState,
  sessionId: string,
): SessionRunsState {
  const current = getOrCreate(state, sessionId);
  if (!current.compactionNotice) return state;
  return {
    bySession: {
      ...state.bySession,
      [sessionId]: {
        ...current,
        compactionNotice: null,
      },
    },
  };
}

export function setCompactionNotice(
  state: SessionRunsState,
  sessionId: string,
  message: string,
): SessionRunsState {
  const current = getOrCreate(state, sessionId);
  return {
    bySession: {
      ...state.bySession,
      [sessionId]: {
        ...current,
        compactionNotice: message,
      },
    },
  };
}
