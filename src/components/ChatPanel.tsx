import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { fuzzyMatch } from "../lib/fuzzy";
import { applyMention, deleteMentionBeforeCursor, detectMention } from "../lib/mention";
import { isVisibleMessage } from "../lib/messages";
import { Message } from "../types";
import { FileMentionPopup } from "./FileMentionPopup";
import { InitCapsule, InitLoadingCapsule } from "./InitCapsule";
import { MessageList } from "./MessageList";
import { SendHintBanner } from "./SendHintBanner";
import { SuggestionCards } from "./SuggestionCards";
import type { SendBlocker } from "../lib/sendReadiness";

interface ChatPanelProps {
  sessionId?: string;
  messages: Message[];
  streamingReasoning: string;
  streamingContent: string;
  activity?: string;
  initializing?: boolean;
  showInitCapsule?: boolean;
  starterSuggestions?: string[];
  followupSuggestions?: string[];
  filePaths?: string[];
  input: string;
  busy: boolean;
  sendHint?: SendBlocker | null;
  onInputChange: (value: string) => void;
  onSend: () => void;
  onInitStarter?: () => void;
  onDismissSendHint?: () => void;
}

export function ChatPanel({
  sessionId,
  messages,
  streamingReasoning,
  streamingContent,
  activity,
  initializing,
  showInitCapsule = false,
  starterSuggestions = [],
  followupSuggestions = [],
  filePaths = [],
  input,
  busy,
  sendHint,
  onInputChange,
  onSend,
  onInitStarter,
  onDismissSendHint,
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

  const showStarterCapsule = showInitCapsule && Boolean(onInitStarter);

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
        <MessageList
          messages={visibleMessages}
          streamingReasoning={streamingReasoning}
          streamingContent={streamingContent}
          activity={activity}
          busy={busy}
        />

        <div ref={bottomRef} className="h-px shrink-0" aria-hidden />
      </div>

      <div className="mt-2 shrink-0 space-y-2">
        {suggestionItems.length > 0 && (
          <SuggestionCards items={suggestionItems} onPick={pickSuggestion} />
        )}

        {sendHint && onDismissSendHint && (
          <SendHintBanner blocker={sendHint} onDismiss={onDismissSendHint} />
        )}

        {(showStarterCapsule || initializing) && (
          <div className="flex flex-wrap items-center gap-1.5">
            {initializing ? (
              <InitLoadingCapsule />
            ) : (
              onInitStarter && <InitCapsule onInit={onInitStarter} />
            )}
          </div>
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
                ? "正在分析文档…"
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
              if (e.key === "Enter" && !e.shiftKey && !inputDisabled && input.trim()) {
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
