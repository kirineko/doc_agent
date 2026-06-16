import { AgentEvent } from "../types";
import {
  applyAgentEvent,
  initialAgentStreamState,
  markAgentBusy,
  markAgentResuming,
  type AgentStreamState,
} from "./agentEvents";

export type SessionRunStatus = "idle" | "running" | "stopping";

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
    event.kind === "context_compacted"
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
