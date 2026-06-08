import { describe, expect, it } from "vitest";
import {
  applyAgentEvent,
  initialAgentStreamState,
  markAgentBusy,
} from "./agentEvents";

describe("applyAgentEvent", () => {
  const sessionId = "session-1";

  it("ignores events from other sessions", () => {
    const next = applyAgentEvent(
      initialAgentStreamState,
      {
        kind: "content_token",
        session_id: "other",
        turn_id: "t1",
        delta: "hello",
      },
      sessionId,
    );
    expect(next.streamingContent).toBe("");
  });

  it("accumulates reasoning and content tokens", () => {
    let state = applyAgentEvent(
      initialAgentStreamState,
      {
        kind: "reasoning_token",
        session_id: sessionId,
        turn_id: "t1",
        delta: "think",
      },
      sessionId,
    );
    state = applyAgentEvent(
      state,
      {
        kind: "content_token",
        session_id: sessionId,
        turn_id: "t1",
        delta: " answer",
      },
      sessionId,
    );
    expect(state.streamingReasoning).toBe("think");
    expect(state.streamingContent).toBe(" answer");
  });

  it("tracks tool call lifecycle", () => {
    let state = markAgentBusy(initialAgentStreamState);
    state = applyAgentEvent(
      state,
      {
        kind: "tool_call",
        session_id: sessionId,
        turn_id: "t1",
        id: "call_1",
        name: "fs_list",
        args: { path: "." },
        status: "running",
      },
      sessionId,
    );
    state = applyAgentEvent(
      state,
      {
        kind: "tool_result",
        session_id: sessionId,
        turn_id: "t1",
        id: "call_1",
        ok: true,
        summary: '{"entries":[]}',
        duration_ms: 8,
      },
      sessionId,
    );

    expect(state.liveTools).toHaveLength(1);
    expect(state.liveTools[0].status).toBe("done");
  });
});
