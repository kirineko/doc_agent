import { Message } from "../types";
import { MessageBubble } from "./MessageBubble";

interface MessageListProps {
  messages: Message[];
  streamingReasoning: string;
  streamingContent: string;
  activity?: string;
  busy: boolean;
}

export function MessageList({
  messages,
  streamingReasoning,
  streamingContent,
  activity,
  busy,
}: MessageListProps) {
  const showStreaming = Boolean(streamingReasoning || streamingContent);

  return (
    <>
      {messages.map((message) => (
        <MessageBubble
          key={message.id}
          role={message.role === "user" ? "user" : "assistant"}
          content={message.content}
          reasoning={message.role === "assistant" ? message.reasoning_content : undefined}
          variant="persisted"
          pending={message.role === "user" && message.id.startsWith("pending-")}
        />
      ))}

      {showStreaming && (
        <MessageBubble
          role="assistant"
          content={streamingContent}
          reasoning={streamingReasoning}
          variant="streaming"
        />
      )}

      {busy && activity && (
        <div className="activity-banner mr-4 flex items-center gap-2 rounded-lg border px-3 py-2 text-xs">
          <span className="h-1.5 w-1.5 shrink-0 animate-pulse rounded-full bg-sky-400" />
          <span className="truncate">{activity}</span>
        </div>
      )}

      {busy && !activity && !showStreaming && (
        <div className="status-idle-banner mr-4 rounded-lg border px-3 py-2 text-xs">
          助手正在回复…
        </div>
      )}
    </>
  );
}
