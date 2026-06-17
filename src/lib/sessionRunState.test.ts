import { describe, expect, it } from "vitest";
import {
  applyEventToSessionRuns,
  countActiveSessionRuns,
  deriveActiveStream,
  initialSessionRunsState,
  isParallelAtCapacity,
  markSessionRunning,
  markSessionStopping,
  MAX_PARALLEL_TURNS,
} from "./sessionRunState";

describe("sessionRunState", () => {
  it("tracks running status per session independently", () => {
    const sessionA = "session-a";
    const sessionB = "session-b";

    let state = markSessionRunning(initialSessionRunsState, sessionA);
    state = applyEventToSessionRuns(state, {
      kind: "content_token",
      session_id: sessionA,
      turn_id: "t1",
      delta: "hello",
    });
    state = markSessionRunning(state, sessionB);
    state = applyEventToSessionRuns(state, {
      kind: "tool_call",
      session_id: sessionB,
      turn_id: "t2",
      id: "call_1",
      name: "fs_list",
      args: { path: "." },
      status: "running",
    });

    expect(state.bySession[sessionA]?.status).toBe("running");
    expect(state.bySession[sessionA]?.streamingContent).toBe("hello");
    expect(state.bySession[sessionB]?.liveTools).toHaveLength(1);
    expect(state.bySession[sessionB]?.status).toBe("running");
  });

  it("derives active stream without clearing background session state", () => {
    const sessionA = "session-a";
    const sessionB = "session-b";

    let state = markSessionRunning(initialSessionRunsState, sessionA);
    state = applyEventToSessionRuns(state, {
      kind: "content_token",
      session_id: sessionA,
      turn_id: "t1",
      delta: "background",
    });
    state = markSessionRunning(state, sessionB);
    state = applyEventToSessionRuns(state, {
      kind: "tool_call",
      session_id: sessionB,
      turn_id: "t2",
      id: "call_1",
      name: "fs_list",
      args: { path: "." },
      status: "running",
    });

    const active = deriveActiveStream(state, sessionB);
    expect(active.busy).toBe(true);
    expect(active.liveTools).toHaveLength(1);
    expect(state.bySession[sessionA]?.streamingContent).toBe("background");
  });

  it("returns idle after turn_cancelled", () => {
    const sessionId = "session-a";
    let state = markSessionRunning(initialSessionRunsState, sessionId);
    state = applyEventToSessionRuns(state, {
      kind: "turn_cancelled",
      session_id: sessionId,
      turn_id: "t1",
    });

    expect(state.bySession[sessionId]?.status).toBe("idle");
    expect(deriveActiveStream(state, sessionId).busy).toBe(false);
  });

  it("marks idle session running when resume emits stream events", () => {
    const sessionId = "session-a";
    const state = applyEventToSessionRuns(initialSessionRunsState, {
      kind: "content_token",
      session_id: sessionId,
      turn_id: "t1",
      delta: "resumed",
    });

    expect(state.bySession[sessionId]?.status).toBe("running");
    expect(deriveActiveStream(state, sessionId).busy).toBe(true);
  });

  it("counts running and stopping sessions toward parallel capacity", () => {
    let state = initialSessionRunsState;
    for (let i = 0; i < MAX_PARALLEL_TURNS; i++) {
      state = markSessionRunning(state, `session-${i}`);
    }
    expect(countActiveSessionRuns(state)).toBe(MAX_PARALLEL_TURNS);
    expect(isParallelAtCapacity(state)).toBe(true);

    state = markSessionStopping(state, "session-0");
    expect(countActiveSessionRuns(state)).toBe(MAX_PARALLEL_TURNS);
    expect(isParallelAtCapacity(state)).toBe(true);

    state = applyEventToSessionRuns(state, {
      kind: "turn_complete",
      session_id: "session-1",
      turn_id: "t1",
    });
    expect(countActiveSessionRuns(state)).toBe(MAX_PARALLEL_TURNS - 1);
    expect(isParallelAtCapacity(state)).toBe(false);
  });
});
