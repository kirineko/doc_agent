import { describe, expect, it } from "vitest";
import {
  applyAgentEvent,
  initialAgentStreamState,
  markAgentBusy,
} from "./agentEvents";
import { appendAssistantStepDone } from "./messages";
import type { AgentEvent } from "../types";
import { makeAssistantMessage, makeUserMessage } from "../test/fixtures/messages";

const sessionId = "session-1";
const turnId = "turn-1";

function apply(events: AgentEvent[]) {
  let stream = markAgentBusy(initialAgentStreamState);
  let messages = [makeUserMessage({ id: "u1", session_id: sessionId, content: "go" })];

  for (const event of events) {
    stream = applyAgentEvent(stream, event, sessionId);
    messages = appendAssistantStepDone(messages, event, sessionId);
  }

  return { stream, messages };
}

describe("multi-step stream flow", () => {
  it("clears streaming between tool-loop steps", () => {
    const { stream, messages } = apply([
      { kind: "reasoning_token", session_id: sessionId, turn_id: turnId, delta: "r1" },
      { kind: "content_token", session_id: sessionId, turn_id: turnId, delta: "c1" },
      {
        kind: "assistant_step_done",
        session_id: sessionId,
        turn_id: turnId,
        message: makeAssistantMessage({
          id: "a1",
          session_id: sessionId,
          content: "",
          reasoning_content: "r1",
        }),
      },
      { kind: "reasoning_token", session_id: sessionId, turn_id: turnId, delta: "r2" },
      { kind: "content_token", session_id: sessionId, turn_id: turnId, delta: "c2" },
      {
        kind: "assistant_step_done",
        session_id: sessionId,
        turn_id: turnId,
        message: makeAssistantMessage({
          id: "a2",
          session_id: sessionId,
          content: "c2",
          reasoning_content: "r2",
        }),
      },
    ]);

    expect(messages.map((m) => m.id)).toEqual(["u1", "a1", "a2"]);
    expect(stream.streamingReasoning).toBe("");
    expect(stream.streamingContent).toBe("");
  });
});
