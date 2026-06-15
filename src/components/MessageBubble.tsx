import { MarkdownView } from "./MarkdownView";
import { MessageAttachments } from "./MessageAttachments";
import type { MessageAttachment } from "../types";

interface MessageBubbleProps {
  role: "user" | "assistant";
  content?: string | null;
  reasoning?: string | null;
  variant: "persisted" | "streaming";
  pending?: boolean;
  attachments?: MessageAttachment[];
  projectId?: string;
  onPreviewImage?: (src: string) => void;
}

export function MessageBubble({
  role,
  content,
  reasoning,
  variant,
  pending,
  attachments = [],
  projectId,
  onPreviewImage,
}: MessageBubbleProps) {
  const isUser = role === "user";
  const reasoningLabel = variant === "streaming" ? "思考中…" : "思考过程";

  const bubbleClass = isUser
    ? `bubble-user ml-8${pending ? " opacity-80" : ""}`
    : variant === "streaming"
      ? "bubble-streaming mr-4"
      : "bubble-assistant mr-4";

  return (
    <div className={`rounded-lg border p-3 ${bubbleClass}`}>
      <div className="mb-1.5 flex items-center gap-2 text-[11px] uppercase tracking-[0.16em] text-fg-muted">
        <span>{isUser ? "你" : "助手"}</span>
        {pending && <span className="normal-case text-link">发送中…</span>}
      </div>
      {reasoning && (
        <details className="reasoning-block mb-2 rounded-md border p-2.5">
          <summary className="cursor-pointer text-[11px]">{reasoningLabel}</summary>
          <div className="reasoning-body mt-2 whitespace-pre-wrap text-sm">{reasoning}</div>
        </details>
      )}
      {isUser && attachments.length > 0 && (
        <MessageAttachments
          attachments={attachments}
          projectId={projectId}
          onPreview={onPreviewImage}
        />
      )}
      {content && <MarkdownView content={content} />}
    </div>
  );
}
