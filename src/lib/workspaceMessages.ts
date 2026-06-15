import type { Message, MessageAttachment } from "../types";

export function createOptimisticUserMessage(
  sessionId: string,
  content: string,
  attachments: MessageAttachment[] = [],
): Message {
  return {
    id: `pending-${crypto.randomUUID()}`,
    session_id: sessionId,
    role: "user",
    content,
    reasoning_content: null,
    tool_call_id: null,
    seq: Number.MAX_SAFE_INTEGER,
    created_at: new Date().toISOString(),
    attachments_json:
      attachments.length > 0 ? JSON.stringify(attachments) : null,
  };
}
