import { MarkdownView } from "./MarkdownView";
import { Message } from "../types";

interface ChatPanelProps {
  messages: Message[];
  streamingReasoning: string;
  streamingContent: string;
  input: string;
  busy: boolean;
  onInputChange: (value: string) => void;
  onSend: () => void;
}

function roleLabel(role: string): string {
  switch (role) {
    case "user":
      return "你";
    case "assistant":
      return "助手";
    default:
      return role;
  }
}

function isVisibleMessage(message: Message): boolean {
  if (message.role === "tool") {
    return false;
  }
  if (message.role === "assistant") {
    const hasContent = Boolean(message.content?.trim());
    const hasReasoning = Boolean(message.reasoning_content?.trim());
    return hasContent || hasReasoning;
  }
  return true;
}

export function ChatPanel({
  messages,
  streamingReasoning,
  streamingContent,
  input,
  busy,
  onInputChange,
  onSend,
}: ChatPanelProps) {
  const visibleMessages = messages.filter(isVisibleMessage);

  return (
    <section className="panel flex min-w-0 flex-1 flex-col p-3">
      <div className="mb-2 text-xs font-medium text-slate-200">会话</div>
      <div className="min-h-0 flex-1 space-y-3 overflow-y-auto pr-2">
        {visibleMessages.map((message) => {
          const isUser = message.role === "user";
          const isPending = message.id.startsWith("pending-");
          return (
            <div
              key={message.id}
              className={`rounded-lg border p-3 ${
                isUser
                  ? `ml-8 border-indigo-900/50 bg-indigo-950/20${isPending ? " opacity-80" : ""}`
                  : "mr-4 border-slate-800 bg-slate-950/60"
              }`}
            >
              <div className="mb-1.5 flex items-center gap-2 text-[11px] uppercase tracking-[0.16em] text-slate-500">
                <span>{roleLabel(message.role)}</span>
                {isPending && <span className="normal-case text-indigo-300">发送中…</span>}
              </div>
              {message.reasoning_content && (
                <details className="mb-2 rounded-md border border-amber-900/40 bg-amber-950/20 p-2.5">
                  <summary className="cursor-pointer text-[11px] text-amber-200">思考过程</summary>
                  <div className="mt-2 whitespace-pre-wrap text-sm text-amber-100/90">
                    {message.reasoning_content}
                  </div>
                </details>
              )}
              {message.content && <MarkdownView content={message.content} />}
            </div>
          );
        })}

        {(streamingReasoning || streamingContent) && (
          <div className="mr-4 rounded-lg border border-indigo-800/60 bg-indigo-950/20 p-3">
            {streamingReasoning && (
              <details className="mb-2 rounded-md border border-amber-900/40 bg-amber-950/20 p-2.5">
                <summary className="cursor-pointer text-[11px] text-amber-200">思考中…</summary>
                <div className="mt-2 whitespace-pre-wrap text-sm text-amber-100/90">{streamingReasoning}</div>
              </details>
            )}
            {streamingContent && <MarkdownView content={streamingContent} />}
          </div>
        )}

        {busy && !streamingReasoning && !streamingContent && (
          <div className="mr-4 rounded-lg border border-slate-800 bg-slate-950/40 px-3 py-2 text-xs text-slate-400">
            助手正在回复…
          </div>
        )}
      </div>

      <div className="mt-3 flex gap-2">
        <textarea
          className="min-h-20 flex-1 resize-none rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm outline-none focus:border-indigo-500 disabled:cursor-not-allowed disabled:opacity-60"
          placeholder={busy ? "等待回复中…" : "输入消息，Enter 发送，Shift+Enter 换行"}
          value={input}
          disabled={busy}
          onChange={(e) => onInputChange(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey && !busy) {
              e.preventDefault();
              onSend();
            }
          }}
        />
        <button
          className="min-w-16 rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium hover:bg-indigo-500 disabled:opacity-50"
          disabled={busy || !input.trim()}
          onClick={onSend}
        >
          {busy ? "发送中" : "发送"}
        </button>
      </div>
    </section>
  );
}
