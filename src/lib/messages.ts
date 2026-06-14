import { AgentEvent, Message } from "../types";

export function isVisibleMessage(message: Message): boolean {
  if (message.role === "user" && message.content?.startsWith("Previous context has been compacted.")) {
    return false;
  }
  if (message.role === "tool") return false;
  if (message.role === "assistant") {
    return Boolean(message.content?.trim()) || Boolean(message.reasoning_content?.trim());
  }
  return true;
}

export function appendMessageDedup(messages: Message[], message: Message): Message[] {
  if (messages.some((item) => item.id === message.id)) {
    return messages;
  }
  return [...messages, message];
}

export function appendAssistantStepDone(
  messages: Message[],
  event: AgentEvent,
  activeSessionId: string | undefined,
): Message[] {
  if (event.kind !== "assistant_step_done" || event.session_id !== activeSessionId) {
    return messages;
  }
  return appendMessageDedup(messages, event.message);
}
