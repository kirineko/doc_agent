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
    expect(state.liveTools[0].summary).toBe('{"entries":[]}');
  });

  it("stores file_busy summary on failed tool result", () => {
    let state = markAgentBusy(initialAgentStreamState);
    const busySummary = JSON.stringify({
      error: "file_busy",
      path: "report.docx",
      message: "当前 report.docx 已被会话「周报」占用，请稍后重试。",
      blocking_session_id: "sess-1",
    });
    state = applyAgentEvent(
      state,
      {
        kind: "tool_call",
        session_id: sessionId,
        turn_id: "t1",
        id: "call_2",
        name: "fs_write",
        args: { path: "report.docx" },
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
        id: "call_2",
        ok: false,
        summary: busySummary,
        duration_ms: 3,
      },
      sessionId,
    );

    expect(state.liveTools[0].status).toBe("error");
    expect(state.liveTools[0].summary).toBe(busySummary);
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
        index: 0,
      },
      sessionId,
    );
    expect(state.liveTools).toHaveLength(1);
    expect(state.liveTools[0].id).toBe("call_1");
    expect(state.liveTools[0].status).toBe("running");
  });

  it("keeps all streaming cards when three pdf_read start running", () => {
    let state = markAgentBusy(initialAgentStreamState);
    for (let index = 0; index < 3; index += 1) {
      state = applyAgentEvent(
        state,
        {
          kind: "tool_call_stream",
          session_id: sessionId,
          turn_id: "t1",
          index,
          name: "pdf_read",
          args_chars: 12,
        },
        sessionId,
      );
    }
    expect(state.liveTools).toHaveLength(3);

    for (let index = 0; index < 3; index += 1) {
      state = applyAgentEvent(
        state,
        {
          kind: "tool_call",
          session_id: sessionId,
          turn_id: "t1",
          id: `call_${index}`,
          name: "pdf_read",
          args: { path: `doc-${index}.pdf` },
          status: "running",
          index,
        },
        sessionId,
      );
    }

    expect(state.liveTools).toHaveLength(3);
    expect(state.liveTools.every((item) => item.status === "running")).toBe(true);
    expect(state.liveTools.map((item) => item.id)).toEqual([
      "call_0",
      "call_1",
      "call_2",
    ]);
  });

  it("fallback tool_call without streaming placeholder keeps other streaming cards", () => {
    let state = markAgentBusy(initialAgentStreamState);
    for (let index = 0; index < 3; index += 1) {
      state = applyAgentEvent(
        state,
        {
          kind: "tool_call_stream",
          session_id: sessionId,
          turn_id: "t1",
          index,
          name: "pdf_read",
          args_chars: 12,
        },
        sessionId,
      );
    }
    state = applyAgentEvent(
      state,
      {
        kind: "tool_call",
        session_id: sessionId,
        turn_id: "t1",
        id: "call_0",
        name: "pdf_read",
        args: { path: "a.pdf" },
        status: "running",
        index: 0,
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
        name: "pdf_read",
        args: { path: "c.pdf" },
        status: "running",
        index: 2,
      },
      sessionId,
    );

    expect(state.liveTools).toHaveLength(3);
    expect(state.liveTools[1]?.id).toBe("streaming-1");
    expect(state.liveTools[1]?.status).toBe("streaming");
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
      compactionNotice: "notice",
    };
    expect(resetAgentStream()).toEqual(initialAgentStreamState);
    expect(resetAgentStream()).not.toEqual(dirty);
  });

  it("ignores context_usage in stream state", () => {
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
    expect(next).toEqual(initialAgentStreamState);
  });

  it("shows compaction notice from context_compacted", () => {
    const next = applyAgentEvent(
      initialAgentStreamState,
      {
        kind: "context_compacted",
        session_id: sessionId,
        before_tokens: 90_000,
        after_tokens: 30_000,
        trigger: "auto",
      },
      sessionId,
    );
    expect(next.compactionNotice).toContain("自动");
  });

  it("shows in-progress notice from compaction_started", () => {
    const auto = applyAgentEvent(
      initialAgentStreamState,
      {
        kind: "compaction_started",
        session_id: sessionId,
        trigger: "auto",
      },
      sessionId,
    );
    expect(auto.compactionNotice).toBe("正在压缩较早的对话历史…");

    const manual = applyAgentEvent(
      initialAgentStreamState,
      {
        kind: "compaction_started",
        session_id: sessionId,
        trigger: "manual",
      },
      sessionId,
    );
    expect(manual.compactionNotice).toBe("正在压缩上下文，请稍候…");
  });

  it("clears in-progress notice on error", () => {
    const busy = applyAgentEvent(
      initialAgentStreamState,
      {
        kind: "compaction_started",
        session_id: sessionId,
        trigger: "auto",
      },
      sessionId,
    );
    const next = applyAgentEvent(
      busy,
      {
        kind: "error",
        session_id: sessionId,
        turn_id: "t1",
        message: "quota exceeded",
      },
      sessionId,
    );
    expect(next.compactionNotice).toBeNull();
  });

  it("shows manual compaction notice", () => {
    const next = applyAgentEvent(
      initialAgentStreamState,
      {
        kind: "context_compacted",
        session_id: sessionId,
        before_tokens: 90_000,
        after_tokens: 30_000,
        trigger: "manual",
      },
      sessionId,
    );
    expect(next.compactionNotice).toContain("手动");
  });
});
