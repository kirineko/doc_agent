import { describe, expect, it } from "vitest";
import { initialAgentStreamState } from "./agentEvents";
import { initialSessionRunsState, sessionRunsReducer } from "./workspaceStream";

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
});
