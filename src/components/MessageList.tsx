import { useMemo } from "react";
import { parseAnsweredClarifyCall } from "../lib/clarifyBrief";
import { Message, ToolCallRecord } from "../types";
import { ClarifyQuestionCard } from "./ClarifyQuestionCard";
import { MessageBubble } from "./MessageBubble";

interface MessageListProps {
  messages: Message[];
  toolCalls?: ToolCallRecord[];
  streamingReasoning: string;
  streamingContent: string;
  activity?: string;
  busy: boolean;
}

export function MessageList({
  messages,
  toolCalls = [],
  streamingReasoning,
  streamingContent,
  activity,
  busy,
}: MessageListProps) {
  const showStreaming = Boolean(streamingReasoning || streamingContent);

  const answeredClarifyByMessage = useMemo(() => {
    type AnsweredClarify = NonNullable<ReturnType<typeof parseAnsweredClarifyCall>>;
    const map = new Map<string, AnsweredClarify[]>();
    for (const call of toolCalls) {
      const parsed = parseAnsweredClarifyCall(call);
      if (!parsed) continue;
      map.set(call.message_id, [...(map.get(call.message_id) ?? []), parsed]);
    }
    return map;
  }, [toolCalls]);

  return (
    <>
      {messages.map((message) => {
        const clarifyCalls = answeredClarifyByMessage.get(message.id) ?? [];
        return (
          <div key={message.id} className="space-y-2">
            <MessageBubble
              role={message.role === "user" ? "user" : "assistant"}
              content={message.content}
              reasoning={message.role === "assistant" ? message.reasoning_content : undefined}
              variant="persisted"
              pending={message.role === "user" && message.id.startsWith("pending-")}
            />
            {clarifyCalls.map(({ callId, question, answer }) => (
              <div key={callId} className="ml-4">
                <ClarifyQuestionCard question={question} answer={answer} />
              </div>
            ))}
          </div>
        );
      })}

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
