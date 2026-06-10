import type { Message } from "../../types";

const DEFAULT_DATE = "2026-01-01";

export function makeUserMessage(
  overrides: Partial<Message> & Pick<Message, "id" | "session_id">,
): Message {
  return {
    role: "user",
    content: "hello",
    reasoning_content: null,
    tool_call_id: null,
    seq: 0,
    created_at: DEFAULT_DATE,
    ...overrides,
  };
}

export function makeAssistantMessage(
  overrides: Partial<Message> & Pick<Message, "id" | "session_id">,
): Message {
  return {
    role: "assistant",
    content: "",
    reasoning_content: null,
    tool_call_id: null,
    seq: 1,
    created_at: DEFAULT_DATE,
    ...overrides,
  };
}
