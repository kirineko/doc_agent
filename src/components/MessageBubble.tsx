import { MarkdownView } from "./MarkdownView";

interface MessageBubbleProps {
  role: "user" | "assistant";
  content?: string | null;
  reasoning?: string | null;
  variant: "persisted" | "streaming";
  pending?: boolean;
}

export function MessageBubble({
  role,
  content,
  reasoning,
  variant,
  pending,
}: MessageBubbleProps) {
  const isUser = role === "user";
  const reasoningLabel = variant === "streaming" ? "思考中…" : "思考过程";

  return (
    <div
      className={`rounded-lg border p-3 ${
        isUser
          ? `ml-8 border-indigo-900/50 bg-indigo-950/20${pending ? " opacity-80" : ""}`
          : variant === "streaming"
            ? "mr-4 border-indigo-800/60 bg-indigo-950/20"
            : "mr-4 border-slate-800 bg-slate-950/60"
      }`}
    >
      <div className="mb-1.5 flex items-center gap-2 text-[11px] uppercase tracking-[0.16em] text-slate-500">
        <span>{isUser ? "你" : "助手"}</span>
        {pending && <span className="normal-case text-indigo-300">发送中…</span>}
      </div>
      {reasoning && (
        <details className="mb-2 rounded-md border border-amber-900/40 bg-amber-950/20 p-2.5">
          <summary className="cursor-pointer text-[11px] text-amber-200">{reasoningLabel}</summary>
          <div className="mt-2 whitespace-pre-wrap text-sm text-amber-100/90">{reasoning}</div>
        </details>
      )}
      {content && <MarkdownView content={content} />}
    </div>
  );
}
