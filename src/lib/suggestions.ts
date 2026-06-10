import type { Message } from "../types";

export function countChatMessages(messages: Message[]): number {
  return messages.filter((m) => m.role === "user" || m.role === "assistant").length;
}

export function shouldRunStarter(
  hasDeepseekKey: boolean,
  messageCount: number,
  alreadyInitializing: boolean,
): boolean {
  return hasDeepseekKey && messageCount === 0 && !alreadyInitializing;
}

/** 迟到的 followup：仅当 user/assistant 消息数已变化时丢弃（用户已发送新消息） */
export function shouldDiscardFollowup(
  requestedMessageCount: number,
  currentMessageCount: number,
): boolean {
  return currentMessageCount !== requestedMessageCount;
}

export function isStaleSessionResult(
  requestSessionId: string,
  activeSessionId: string | undefined,
): boolean {
  return requestSessionId !== activeSessionId;
}
