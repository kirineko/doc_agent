import { describe, expect, it } from "vitest";
import {
  applyAgentEvent,
  initialAgentStreamState,
  markAgentBusy,
  markAgentResuming,
  resetAgentStream,
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

  it("shows streaming placeholder while tool args are being generated", () => {
    let state = markAgentBusy(initialAgentStreamState);
    state = applyAgentEvent(
      state,
      {
        kind: "tool_call_stream",
        session_id: sessionId,
        turn_id: "t1",
        index: 0,
        name: "skill_run",
        args_chars: 1200,
      },
      sessionId,
    );

    expect(state.liveTools).toHaveLength(1);
    expect(state.liveTools[0].status).toBe("streaming");
    expect(state.liveTools[0].argsChars).toBe(1200);

    // 进度更新覆盖同一占位条目
    state = applyAgentEvent(
      state,
      {
        kind: "tool_call_stream",
        session_id: sessionId,
        turn_id: "t1",
        index: 0,
        name: "skill_run",
        args_chars: 4800,
      },
      sessionId,
    );
    expect(state.liveTools).toHaveLength(1);
    expect(state.liveTools[0].argsChars).toBe(4800);

    // 真实调用开始后占位条目被移除
    state = applyAgentEvent(
      state,
      {
        kind: "tool_call",
        session_id: sessionId,
        turn_id: "t1",
        id: "call_1",
        name: "skill_run",
        args: { code: "..." },
        status: "running",
      },
      sessionId,
    );
    expect(state.liveTools).toHaveLength(1);
    expect(state.liveTools[0].id).toBe("call_1");
    expect(state.liveTools[0].status).toBe("running");
  });

  it("clears streaming buffers on assistant_step_done", () => {
    let state = markAgentBusy(initialAgentStreamState);
    state = applyAgentEvent(
      state,
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
    state = applyAgentEvent(
      state,
      {
        kind: "assistant_step_done",
        session_id: sessionId,
        turn_id: "t1",
        message: {
          id: "m1",
          session_id: sessionId,
          role: "assistant",
          content: " answer",
          reasoning_content: "think",
          tool_call_id: null,
          seq: 1,
          created_at: "2026-01-01",
        },
      },
      sessionId,
    );
    expect(state.streamingReasoning).toBe("");
    expect(state.streamingContent).toBe("");
    expect(state.busy).toBe(true);
  });

  it("ignores assistant_step_done from other sessions", () => {
    const state = applyAgentEvent(
      {
        ...initialAgentStreamState,
        streamingContent: "partial",
      },
      {
        kind: "assistant_step_done",
        session_id: "other",
        turn_id: "t1",
        message: {
          id: "m1",
          session_id: "other",
          role: "assistant",
          content: "x",
          reasoning_content: null,
          tool_call_id: null,
          seq: 1,
          created_at: "2026-01-01",
        },
      },
      sessionId,
    );
    expect(state.streamingContent).toBe("partial");
  });

  it("markAgentResuming keeps liveTools while clearing streaming buffers", () => {
    let state = markAgentBusy(initialAgentStreamState);
    state = applyAgentEvent(
      state,
      {
        kind: "tool_call",
        session_id: sessionId,
        turn_id: "t1",
        id: "call_1",
        name: "skill_read",
        args: { skill: "clarify" },
        status: "done",
      },
      sessionId,
    );
    state = applyAgentEvent(
      state,
      {
        kind: "tool_call",
        session_id: sessionId,
        turn_id: "t1",
        id: "call_2",
        name: "clarify_ask",
        args: { id: "q1" },
        status: "awaiting_user",
      },
      sessionId,
    );
    state = {
      ...state,
      streamingReasoning: "thinking",
      streamingContent: "partial",
    };

    const resumed = markAgentResuming(state);

    expect(resumed.busy).toBe(true);
    expect(resumed.liveTools).toHaveLength(2);
    expect(resumed.liveTools[1]?.status).toBe("awaiting_user");
    expect(resumed.streamingReasoning).toBe("");
    expect(resumed.streamingContent).toBe("");
  });

  it("resetAgentStream clears all stream state", () => {
    const dirty = {
      ...markAgentBusy(initialAgentStreamState),
      liveTools: [
        { id: "call_1", name: "fs_list", args: {}, status: "done" },
      ],
      contextRatio: 0.42,
      compactionNotice: "notice",
    };
    expect(resetAgentStream()).toEqual(initialAgentStreamState);
    expect(resetAgentStream()).not.toEqual(dirty);
  });

  it("updates context ratio from context_usage", () => {
    const next = applyAgentEvent(
      initialAgentStreamState,
      {
        kind: "context_usage",
        session_id: sessionId,
        used_tokens: 42_000,
        max_tokens: 100_000,
        ratio: 0.42,
      },
      sessionId,
    );
    expect(next.contextRatio).toBe(0.42);
  });

  it("shows compaction notice from context_compacted", () => {
    const next = applyAgentEvent(
      initialAgentStreamState,
      {
        kind: "context_compacted",
        session_id: sessionId,
        before_tokens: 90_000,
        after_tokens: 30_000,
      },
      sessionId,
    );
    expect(next.compactionNotice).toContain("压缩");
  });
});
