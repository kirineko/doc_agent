import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { fuzzyMatch } from "../lib/fuzzy";
import { applyMention, deleteMentionBeforeCursor, detectMention } from "../lib/mention";
import { Message } from "../types";
import { FileMentionPopup } from "./FileMentionPopup";
import { MarkdownView } from "./MarkdownView";
import { SuggestionCards } from "./SuggestionCards";

interface ChatPanelProps {
  sessionId?: string;
  messages: Message[];
  streamingReasoning: string;
  streamingContent: string;
  activity?: string;
  initializing?: boolean;
  starterSuggestions?: string[];
  followupSuggestions?: string[];
  filePaths?: string[];
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
  if (message.role === "tool") return false;
  if (message.role === "assistant") {
    return Boolean(message.content?.trim()) || Boolean(message.reasoning_content?.trim());
  }
  return true;
}

export function ChatPanel({
  sessionId,
  messages,
  streamingReasoning,
  streamingContent,
  activity,
  initializing,
  starterSuggestions = [],
  followupSuggestions = [],
  filePaths = [],
  input,
  busy,
  onInputChange,
  onSend,
}: ChatPanelProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const bottomRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const stickToBottomRef = useRef(true);
  const [mentionIndex, setMentionIndex] = useState(0);
  const [cursor, setCursor] = useState(0);

  const visibleMessages = useMemo(() => messages.filter(isVisibleMessage), [messages]);
  const lastMessageId = visibleMessages.at(-1)?.id;
  const inputDisabled = busy || initializing;
  const mention = detectMention(input, cursor);
  const mentionMatches = useMemo(() => {
    if (!mention || filePaths.length === 0) return [];
    return fuzzyMatch(mention.query, filePaths).slice(0, 8);
  }, [mention, filePaths]);
  const suggestionItems = useMemo(() => {
    if (initializing) return [];
    if (visibleMessages.length === 0) return starterSuggestions;
    if (busy) return [];
    return followupSuggestions;
  }, [initializing, busy, visibleMessages.length, starterSuggestions, followupSuggestions]);

  const scrollToBottom = (behavior: ScrollBehavior = "auto") => {
    bottomRef.current?.scrollIntoView({ behavior, block: "end" });
  };

  const handleScroll = () => {
    const el = scrollRef.current;
    if (!el) return;
    stickToBottomRef.current = el.scrollHeight - el.scrollTop - el.clientHeight < 72;
  };

  useEffect(() => {
    stickToBottomRef.current = true;
    setMentionIndex(0);
  }, [sessionId]);

  useEffect(() => {
    if (busy) stickToBottomRef.current = true;
  }, [busy]);

  useLayoutEffect(() => {
    if (!stickToBottomRef.current) return;
    const instant = Boolean(
      streamingReasoning || streamingContent || busy || initializing || suggestionItems.length > 0,
    );
    scrollToBottom(instant ? "auto" : "smooth");
  }, [
    lastMessageId,
    streamingReasoning,
    streamingContent,
    activity,
    busy,
    initializing,
    suggestionItems.length,
  ]);

  useEffect(() => {
    setMentionIndex(0);
  }, [mention?.query]);

  function focusTextareaAt(pos: number) {
    requestAnimationFrame(() => {
      textareaRef.current?.focus();
      textareaRef.current?.setSelectionRange(pos, pos);
    });
  }

  function pickMention(path: string) {
    if (!mention) return;
    const result = applyMention(input, mention, path);
    onInputChange(result.text);
    setCursor(result.cursor);
    focusTextareaAt(result.cursor);
  }

  function pickSuggestion(text: string) {
    onInputChange(text);
    setCursor(text.length);
    focusTextareaAt(text.length);
  }

  return (
    <section className="panel flex min-w-0 flex-1 flex-col p-3">
      <div className="mb-2 text-xs font-medium text-slate-200">会话</div>
      <div
        ref={scrollRef}
        onScroll={handleScroll}
        className="min-h-0 flex-1 space-y-3 overflow-y-auto pr-2"
      >
        {initializing && (
          <div className="flex flex-col items-center gap-2 py-12 text-sm text-slate-400">
            <span className="h-2 w-2 animate-pulse rounded-full bg-indigo-400" />
            会话初始化中…正在阅读项目文档
          </div>
        )}

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
                <div className="mt-2 whitespace-pre-wrap text-sm text-amber-100/90">
                  {streamingReasoning}
                </div>
              </details>
            )}
            {streamingContent && <MarkdownView content={streamingContent} />}
          </div>
        )}

        {busy && activity && (
          <div className="mr-4 flex items-center gap-2 rounded-lg border border-sky-900/50 bg-sky-950/20 px-3 py-2 text-xs text-sky-200">
            <span className="h-1.5 w-1.5 shrink-0 animate-pulse rounded-full bg-sky-400" />
            <span className="truncate">{activity}</span>
          </div>
        )}

        {busy && !activity && !streamingReasoning && !streamingContent && (
          <div className="mr-4 rounded-lg border border-slate-800 bg-slate-950/40 px-3 py-2 text-xs text-slate-400">
            助手正在回复…
          </div>
        )}

        <div ref={bottomRef} className="h-px shrink-0" aria-hidden />
      </div>

      <div className="mt-2 shrink-0 space-y-2">
        {suggestionItems.length > 0 && (
          <SuggestionCards items={suggestionItems} onPick={pickSuggestion} />
        )}

        <div className="relative flex gap-2">
        {mention && filePaths.length > 0 && (
          <FileMentionPopup
            matches={mentionMatches}
            selectedIndex={mentionIndex}
            onPick={pickMention}
          />
        )}
        <textarea
          ref={textareaRef}
          className="min-h-20 flex-1 resize-none rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm outline-none focus:border-indigo-500 disabled:cursor-not-allowed disabled:opacity-60"
          placeholder={
            initializing
              ? "会话初始化中…"
              : busy
                ? "等待回复中…"
                : "输入消息，Enter 发送，Shift+Enter 换行，@ 引用文件"
          }
          value={input}
          disabled={inputDisabled}
          onChange={(e) => {
            onInputChange(e.target.value);
            setCursor(e.target.selectionStart);
          }}
          onSelect={(e) => setCursor(e.currentTarget.selectionStart)}
          onKeyDown={(e) => {
            if (e.key === "Backspace" && !e.shiftKey) {
              const deleted = deleteMentionBeforeCursor(input, cursor);
              if (deleted) {
                e.preventDefault();
                onInputChange(deleted.text);
                setCursor(deleted.cursor);
                focusTextareaAt(deleted.cursor);
                return;
              }
            }
            if (mention && mentionMatches.length > 0) {
              const count = mentionMatches.length;
              if (e.key === "ArrowDown") {
                e.preventDefault();
                setMentionIndex((i) => (i + 1) % count);
                return;
              }
              if (e.key === "ArrowUp") {
                e.preventDefault();
                setMentionIndex((i) => (i - 1 + count) % count);
                return;
              }
              if (e.key === "Enter" || e.key === "Tab") {
                e.preventDefault();
                pickMention(mentionMatches[mentionIndex]?.item ?? mentionMatches[0]!.item);
                return;
              }
            }
            if (e.key === "Enter" && !e.shiftKey && !inputDisabled) {
              e.preventDefault();
              onSend();
            }
          }}
        />
        <button
          className="min-w-16 rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium hover:bg-indigo-500 disabled:opacity-50"
          disabled={inputDisabled || !input.trim()}
          onClick={onSend}
        >
          {busy ? "发送中" : "发送"}
        </button>
        </div>
      </div>
    </section>
  );
}
