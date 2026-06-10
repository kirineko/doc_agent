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
        <div className="mr-4 flex items-center gap-2 rounded-lg border border-sky-900/50 bg-sky-950/20 px-3 py-2 text-xs text-sky-200">
          <span className="h-1.5 w-1.5 shrink-0 animate-pulse rounded-full bg-sky-400" />
          <span className="truncate">{activity}</span>
        </div>
      )}

      {busy && !activity && !showStreaming && (
        <div className="mr-4 rounded-lg border border-slate-800 bg-slate-950/40 px-3 py-2 text-xs text-slate-400">
          助手正在回复…
        </div>
      )}
    </>
  );
}
