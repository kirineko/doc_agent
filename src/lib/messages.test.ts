import { describe, expect, it } from "vitest";
import {
  appendAssistantStepDone,
  appendMessageDedup,
  isVisibleMessage,
} from "./messages";
import type { AgentEvent } from "../types";
import { makeAssistantMessage, makeUserMessage } from "../test/fixtures/messages";

describe("messages helpers", () => {
  const base = makeAssistantMessage({
    id: "m1",
    session_id: "s1",
    content: "hello",
  });

  it("appends new messages", () => {
    const next = appendMessageDedup([], base);
    expect(next).toHaveLength(1);
    expect(next[0]?.id).toBe("m1");
  });

  it("deduplicates by id", () => {
    const next = appendMessageDedup([base], { ...base, content: "updated" });
    expect(next).toHaveLength(1);
    expect(next[0]?.content).toBe("hello");
  });

  it("ignores duplicate assistant_step_done appends", () => {
    const a1 = makeAssistantMessage({ id: "a1", session_id: "s1", content: "", reasoning_content: "r1" });
    const a2 = makeAssistantMessage({ id: "a2", session_id: "s1", content: "answer", reasoning_content: "r2" });
    let messages = appendMessageDedup([makeUserMessage({ id: "u1", session_id: "s1", content: "go" })], a1);
    messages = appendMessageDedup(messages, a2);

    expect(appendMessageDedup(messages, a2)).toEqual(messages);
  });

  it("appendAssistantStepDone gates by session and event kind", () => {
    const event: AgentEvent = {
      kind: "assistant_step_done",
      session_id: "s1",
      turn_id: "t1",
      message: base,
    };
    const messages = [makeUserMessage({ id: "u1", session_id: "s1" })];

    expect(appendAssistantStepDone(messages, event, "s1")).toHaveLength(2);
    expect(appendAssistantStepDone(messages, event, "other")).toBe(messages);
    expect(
      appendAssistantStepDone(messages, { kind: "turn_complete", session_id: "s1", turn_id: "t1" }, "s1"),
    ).toBe(messages);
  });

  it("isVisibleMessage shows user messages with attachments but no text", () => {
    expect(
      isVisibleMessage(
        makeUserMessage({
          id: "u3",
          session_id: "s1",
          content: "",
          attachments_json: JSON.stringify([{ path: ".cache/attachments/a.png", mime: "image/png" }]),
        }),
      ),
    ).toBe(true);
  });

  it("isVisibleMessage hides tool messages and empty assistants", () => {
    expect(isVisibleMessage(makeUserMessage({ id: "u1", session_id: "s1" }))).toBe(true);
    expect(
      isVisibleMessage({
        ...makeAssistantMessage({ id: "a1", session_id: "s1" }),
        content: "",
        reasoning_content: null,
      }),
    ).toBe(false);
    expect(
      isVisibleMessage({
        ...makeAssistantMessage({ id: "a2", session_id: "s1" }),
        content: "",
        reasoning_content: "thinking",
      }),
    ).toBe(true);
    expect(
      isVisibleMessage(
        makeUserMessage({
          id: "u2",
          session_id: "s1",
          content: "Previous context has been compacted. Continue from this summary:\n\nfoo",
        }),
      ),
    ).toBe(false);
  });
});
