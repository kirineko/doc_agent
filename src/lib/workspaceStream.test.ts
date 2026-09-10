import { describe, expect, it } from "vitest";
import { initialAgentStreamState } from "./agentEvents";
import { initialSessionRunsState, sessionRunsReducer } from "./workspaceStream";
import type { SessionRunsState } from "./sessionRunState";

function withArtifact(sessionId: string): SessionRunsState {
  let state = sessionRunsReducer(initialSessionRunsState, { type: "busy", sessionId });
  state = sessionRunsReducer(state, {
    type: "event",
    event: {
      kind: "tool_call",
      session_id: sessionId,
      turn_id: "t1",
      id: "call_1",
      name: "fs_write",
      args: { path: "report.docx" },
      status: "running",
    },
  });
  state = sessionRunsReducer(state, {
    type: "event",
    event: {
      kind: "tool_result",
      session_id: sessionId,
      turn_id: "t1",
      id: "call_1",
      ok: true,
      summary: "done",
      duration_ms: 12,
      changed_paths: ["report.docx"],
    },
  });
  return state;
}

describe("sessionRunsReducer", () => {
  it("marks session running on busy action", () => {
    const next = sessionRunsReducer(initialSessionRunsState, {
      type: "busy",
      sessionId: "s1",
    });
    expect(next.bySession.s1?.status).toBe("running");
  });

  it("applies turn_complete event to force idle", () => {
    let state = sessionRunsReducer(initialSessionRunsState, {
      type: "busy",
      sessionId: "s1",
    });
    state = sessionRunsReducer(state, {
      type: "event",
      event: {
        kind: "turn_complete",
        session_id: "s1",
        turn_id: "t1",
      },
    });
    expect(state.bySession.s1?.status).toBe("idle");
  });

  it("clears retry notice when cancelled during backoff", () => {
    let state = sessionRunsReducer(initialSessionRunsState, {
      type: "busy",
      sessionId: "s1",
    });
    state = sessionRunsReducer(state, {
      type: "event",
      event: {
        kind: "provider_retry",
        session_id: "s1",
        turn_id: "t1",
        attempt: 1,
        max: 2,
        code: "network",
        delay_ms: 1000,
      },
    });
    expect(state.bySession.s1?.retryNotice).not.toBeNull();
    state = sessionRunsReducer(state, {
      type: "event",
      event: { kind: "turn_cancelled", session_id: "s1", turn_id: "t1" },
    });
    expect(state.bySession.s1?.status).toBe("idle");
    expect(state.bySession.s1?.busy).toBe(false);
    expect(state.bySession.s1?.retryNotice).toBeNull();
  });

  it("shows invoke reject as a turn error card payload", () => {
    const next = sessionRunsReducer(initialSessionRunsState, {
      type: "event",
      event: {
        kind: "error",
        session_id: "s1",
        turn_id: "local",
        message: "消息不能为空",
      },
    });
    expect(next.bySession.s1?.turnError?.message).toBe("消息不能为空");
    expect(next.bySession.s1?.status).toBe("idle");
    // 新消息进入 busy 即清掉上一轮错误卡片
    const cleared = sessionRunsReducer(next, { type: "busy", sessionId: "s1" });
    expect(cleared.bySession.s1?.turnError).toBeNull();
  });

  it("clears compaction notice for session", () => {
    const state = sessionRunsReducer(
      {
        bySession: {
          s1: {
            ...initialAgentStreamState,
            status: "idle",
            compactionNotice: "已压缩历史",
          },
        },
      },
      { type: "clear_compaction_notice", sessionId: "s1" },
    );
    expect(state.bySession.s1?.compactionNotice).toBeNull();
  });

  it("clears turnArtifacts on a real busy turn", () => {
    const seeded = withArtifact("s1");
    expect(seeded.bySession.s1?.turnArtifacts).toHaveLength(1);

    const next = sessionRunsReducer(seeded, { type: "busy", sessionId: "s1" });
    expect(next.bySession.s1?.turnArtifacts).toHaveLength(0);
  });

  it("preserves turnArtifacts during manual compact (busy_compact)", () => {
    const seeded = withArtifact("s1");
    expect(seeded.bySession.s1?.turnArtifacts).toHaveLength(1);

    const next = sessionRunsReducer(seeded, { type: "busy_compact", sessionId: "s1" });
    expect(next.bySession.s1?.status).toBe("running");
    expect(next.bySession.s1?.busy).toBe(true);
    expect(next.bySession.s1?.turnArtifacts).toHaveLength(1);
    expect(next.bySession.s1?.turnArtifacts[0]?.path).toBe("report.docx");
  });
});
