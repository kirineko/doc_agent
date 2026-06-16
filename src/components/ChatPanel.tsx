import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import {
  readClipboardImageFile,
  type PendingAttachment,
} from "../lib/attachments";
import type { MentionFileEntry } from "../lib/projectFiles";
import { orderMentionFileMatchesForDisplay, searchMentionFiles } from "../lib/mentionFiles";
import { applyMention, deleteMentionBeforeCursor, detectMention, expandMentionDirectory } from "../lib/mention";
import { isVisibleMessage } from "../lib/messages";
import { applySlash, detectSlash } from "../lib/slash";
import { deletePlaceholderAtCursor, deletePlaceholderBeforeCursor } from "../lib/promptPlaceholder";
import { SLASH_COMMANDS } from "../lib/slashCommands";
import { flattenSlashGroups, searchSlashCommands } from "../lib/slashFuzzy";
import { ClarifyQuestion, Message, ToolCallRecord } from "../types";
import { ContextUsageIndicator } from "./ContextUsageIndicator";
import { ClarifyQuestionCard } from "./ClarifyQuestionCard";
import { FileMentionPopup } from "./FileMentionPopup";
import { InitCapsule, InitLoadingCapsule } from "./InitCapsule";
import { ImagePreviewOverlay } from "./ImagePreviewOverlay";
import { MessageList } from "./MessageList";
import { PendingAttachmentChips } from "./PendingAttachmentChips";
import { SendHintBanner } from "./SendHintBanner";
import { SlashCommandPopup } from "./SlashCommandPopup";
import { SuggestionCards } from "./SuggestionCards";
import type { SendBlocker } from "../lib/sendReadiness";

interface ChatPanelProps {
  sessionId?: string;
  messages: Message[];
  toolCalls?: ToolCallRecord[];
  activeClarify?: { question: ClarifyQuestion };
  streamingReasoning: string;
  streamingContent: string;
  activity?: string;
  initializing?: boolean;
  showInitCapsule?: boolean;
  starterSuggestions?: string[];
  followupSuggestions?: string[];
  fileEntries?: MentionFileEntry[];
  input: string;
  busy: boolean;
  contextRatio?: number;
  compactionNotice?: string | null;
  sendHint?: SendBlocker | null;
  pendingAttachments: PendingAttachment[];
  visionToast?: string | null;
  projectId?: string;
  onInputChange: (value: string) => void;
  onSend: () => void;
  onPasteImage: (file: File, mime: string) => void | Promise<void>;
  onRemoveAttachment: (path: string) => void;
  onDismissVisionToast?: () => void;
  onSubmitClarify?: (payload: { selected: string[]; custom?: string | null }) => void;
  onInitStarter?: () => void;
  onDismissSendHint?: () => void;
  onDismissCompactionNotice?: () => void;
}

export function ChatPanel({
  sessionId,
  messages,
  toolCalls = [],
  activeClarify,
  streamingReasoning,
  streamingContent,
  activity,
  initializing,
  showInitCapsule = false,
  starterSuggestions = [],
  followupSuggestions = [],
  fileEntries = [],
  input,
  busy,
  contextRatio,
  compactionNotice,
  sendHint,
  pendingAttachments,
  visionToast,
  projectId,
  onInputChange,
  onSend,
  onPasteImage,
  onRemoveAttachment,
  onDismissVisionToast,
  onSubmitClarify,
  onInitStarter,
  onDismissSendHint,
  onDismissCompactionNotice,
}: ChatPanelProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const bottomRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const stickToBottomRef = useRef(true);
  const [mentionIndex, setMentionIndex] = useState(0);
  const [mentionDismissed, setMentionDismissed] = useState(false);
  const [slashIndex, setSlashIndex] = useState(0);
  const [slashDismissed, setSlashDismissed] = useState(false);
  const [cursor, setCursor] = useState(0);
  const [previewImageSrc, setPreviewImageSrc] = useState<string | null>(null);

  const visibleMessages = useMemo(() => messages.filter(isVisibleMessage), [messages]);
  const lastMessageId = visibleMessages.at(-1)?.id;
  const hasActiveClarify = Boolean(activeClarify);
  const inputDisabled = busy || initializing || hasActiveClarify;
  const canSend = Boolean(input.trim() || pendingAttachments.length > 0);
  const mention = detectMention(input, cursor);
  const slash = mention ? null : detectSlash(input, cursor);
  const mentionMatches = useMemo(() => {
    if (!mention || fileEntries.length === 0) return [];
    return searchMentionFiles(mention.query, fileEntries);
  }, [mention, fileEntries]);
  const mentionDisplayMatches = useMemo(() => {
    if (!mention) return [];
    return orderMentionFileMatchesForDisplay(mentionMatches, mention.query);
  }, [mention, mentionMatches]);
  const slashGroups = useMemo(
    () => (slash ? searchSlashCommands(slash.query) : []),
    [slash?.query],
  );
  const slashMatches = useMemo(() => flattenSlashGroups(slashGroups), [slashGroups]);
  const showSlashPopup = Boolean(slash && !slashDismissed && !inputDisabled);
  const showMentionPopup = Boolean(mention && !mentionDismissed && fileEntries.length > 0 && !inputDisabled);
  const suggestionItems = useMemo(() => {
    if (initializing) return [];
    if (hasActiveClarify) return [];
    if (visibleMessages.length === 0) return starterSuggestions;
    if (busy) return [];
    return followupSuggestions;
  }, [initializing, hasActiveClarify, busy, visibleMessages.length, starterSuggestions, followupSuggestions]);

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
    setMentionDismissed(false);
    setSlashIndex(0);
    setSlashDismissed(false);
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
    pendingAttachments.length,
  ]);

  useEffect(() => {
    setMentionIndex(0);
    setMentionDismissed(false);
  }, [mention?.query]);

  useEffect(() => {
    setSlashIndex(0);
    setSlashDismissed(false);
  }, [slash?.query]);

  useEffect(() => {
    if (!compactionNotice) return;
    const timer = window.setTimeout(() => {
      onDismissCompactionNotice?.();
    }, 5000);
    return () => window.clearTimeout(timer);
  }, [compactionNotice, onDismissCompactionNotice]);

  useEffect(() => {
    if (!visionToast) return;
    const timer = window.setTimeout(() => {
      onDismissVisionToast?.();
    }, 5000);
    return () => window.clearTimeout(timer);
  }, [visionToast, onDismissVisionToast]);

  function focusTextareaAt(start: number, end = start) {
    requestAnimationFrame(() => {
      textareaRef.current?.focus();
      textareaRef.current?.setSelectionRange(start, end);
    });
  }

  function pickMention(path: string) {
    if (!mention) return;
    const result = applyMention(input, mention, path);
    onInputChange(result.text);
    setCursor(result.cursor);
    focusTextareaAt(result.cursor);
  }

  function browseMentionDirectory(path: string) {
    if (!mention) return;
    const result = expandMentionDirectory(input, mention, path);
    onInputChange(result.text);
    setCursor(result.cursor);
    setMentionIndex(0);
    focusTextareaAt(result.cursor);
  }

  function pickSlash(commandId: string) {
    if (!slash) return;
    const command = SLASH_COMMANDS.find((item) => item.id === commandId);
    if (!command) return;
    const result = applySlash(input, slash, command.prompt);
    onInputChange(result.text);
    setCursor(result.selectionEnd);
    setSlashDismissed(false);
    focusTextareaAt(result.cursor, result.selectionEnd);
  }

  function pickSuggestion(text: string) {
    onInputChange(text);
    setCursor(text.length);
    focusTextareaAt(text.length);
  }

  async function handlePaste(event: React.ClipboardEvent<HTMLTextAreaElement>) {
    const image = readClipboardImageFile(event.clipboardData);
    if (!image) return;
    event.preventDefault();
    await onPasteImage(image.file, image.mime);
  }

  return (
    <section className="panel flex h-full min-h-0 flex-col p-3">
      <div className="mb-2 flex items-center justify-between gap-2">
        <div className="text-xs font-medium text-fg-heading">会话</div>
        <ContextUsageIndicator ratio={contextRatio ?? 0} hidden={!projectId} />
      </div>
      <div
        ref={scrollRef}
        onScroll={handleScroll}
        className="min-h-0 flex-1 space-y-3 overflow-y-auto pr-2"
      >
        <MessageList
          messages={visibleMessages}
          toolCalls={toolCalls}
          streamingReasoning={streamingReasoning}
          streamingContent={streamingContent}
          activity={activity}
          busy={busy}
          projectId={projectId}
          onPreviewImage={setPreviewImageSrc}
        />

        <div ref={bottomRef} className="h-px shrink-0" aria-hidden />
      </div>

      <div className="mt-2 shrink-0 space-y-2">
        {suggestionItems.length > 0 && (
          <SuggestionCards items={suggestionItems} onPick={pickSuggestion} />
        )}

        {activeClarify && (
          <ClarifyQuestionCard
            question={activeClarify.question}
            onSubmit={onSubmitClarify}
          />
        )}

        {sendHint && onDismissSendHint && (
          <SendHintBanner blocker={sendHint} onDismiss={onDismissSendHint} />
        )}

        {visionToast && (
          <div className="flex items-start justify-between gap-2 rounded-lg border border-amber-600/40 bg-amber-500/10 px-3 py-2 text-xs text-amber-700 dark:text-amber-300">
            <span>{visionToast}</span>
            {onDismissVisionToast && (
              <button
                type="button"
                className="shrink-0 text-fg-muted hover:text-fg"
                onClick={onDismissVisionToast}
              >
                ×
              </button>
            )}
          </div>
        )}

        {compactionNotice && (
          <div className="rounded-lg border border-border bg-surface px-3 py-2 text-xs text-fg-secondary">
            {compactionNotice}
          </div>
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

        <PendingAttachmentChips
          items={pendingAttachments}
          disabled={inputDisabled}
          onRemove={onRemoveAttachment}
          onPreview={setPreviewImageSrc}
        />

        <div className="relative flex gap-2">
          {showMentionPopup && mention && (
            <FileMentionPopup
              query={mention.query}
              matches={mentionMatches}
              selectedIndex={mentionIndex}
              onPick={pickMention}
            />
          )}
          {showSlashPopup && (
            <SlashCommandPopup
              groups={slashGroups}
              selectedIndex={slashIndex}
              onPick={pickSlash}
            />
          )}
          <textarea
            ref={textareaRef}
            className="input-field min-h-20 flex-1 resize-none rounded-lg px-3 py-2 text-sm disabled:cursor-not-allowed disabled:opacity-60"
            placeholder={
              initializing
                ? "正在分析文档…"
                : hasActiveClarify
                  ? "请先回答上方澄清问题"
                  : busy
                  ? "等待回复中…"
                  : "输入消息，Enter 发送，Shift+Enter 换行，@ 引用文件，/ 选择任务，可粘贴图片"
            }
            value={input}
            disabled={inputDisabled}
            onPaste={(e) => void handlePaste(e)}
            onChange={(e) => {
              onInputChange(e.target.value);
              setCursor(e.target.selectionStart);
            }}
            onSelect={(e) => setCursor(e.currentTarget.selectionStart)}
            onKeyDown={(e) => {
              if (e.key === "Backspace" && !e.shiftKey) {
                const deletedPh = deletePlaceholderBeforeCursor(input, cursor);
                if (deletedPh) {
                  e.preventDefault();
                  onInputChange(deletedPh.text);
                  setCursor(deletedPh.cursor);
                  focusTextareaAt(deletedPh.cursor);
                  return;
                }
                const deleted = deleteMentionBeforeCursor(input, cursor);
                if (deleted) {
                  e.preventDefault();
                  onInputChange(deleted.text);
                  setCursor(deleted.cursor);
                  focusTextareaAt(deleted.cursor);
                  return;
                }
              }
              if (e.key === "Delete" && !e.shiftKey) {
                const deletedPh = deletePlaceholderAtCursor(input, cursor);
                if (deletedPh) {
                  e.preventDefault();
                  onInputChange(deletedPh.text);
                  setCursor(deletedPh.cursor);
                  focusTextareaAt(deletedPh.cursor);
                  return;
                }
              }
              if (mention && !mentionDismissed) {
                if (e.key === "Escape") {
                  e.preventDefault();
                  setMentionDismissed(true);
                  return;
                }
                if (mentionDisplayMatches.length > 0) {
                  const count = mentionDisplayMatches.length;
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
                  if (e.key === "Enter") {
                    e.preventDefault();
                    pickMention(
                      mentionDisplayMatches[mentionIndex]?.item ?? mentionDisplayMatches[0]!.item,
                    );
                    return;
                  }
                  if (e.key === "Tab") {
                    e.preventDefault();
                    const selected = mentionDisplayMatches[mentionIndex] ?? mentionDisplayMatches[0]!;
                    if (selected.isDir) {
                      browseMentionDirectory(selected.item);
                    } else {
                      pickMention(selected.item);
                    }
                    return;
                  }
                }
              }
              if (showSlashPopup) {
                if (e.key === "Escape") {
                  e.preventDefault();
                  setSlashDismissed(true);
                  return;
                }
                if (slashMatches.length > 0) {
                  const count = slashMatches.length;
                  if (e.key === "ArrowDown") {
                    e.preventDefault();
                    setSlashIndex((i) => (i + 1) % count);
                    return;
                  }
                  if (e.key === "ArrowUp") {
                    e.preventDefault();
                    setSlashIndex((i) => (i - 1 + count) % count);
                    return;
                  }
                  if (e.key === "Enter" || e.key === "Tab") {
                    e.preventDefault();
                    const match = slashMatches[slashIndex] ?? slashMatches[0]!;
                    pickSlash(match.command.id);
                    return;
                  }
                } else if (e.key === "Enter" || e.key === "Tab") {
                  e.preventDefault();
                  return;
                }
              }
              if (e.key === "Enter" && !e.shiftKey && !inputDisabled && canSend) {
                e.preventDefault();
                onSend();
              }
            }}
          />
          <button
            className="btn-primary min-w-16 rounded-lg px-4 py-2 text-sm font-medium"
            disabled={inputDisabled || !canSend}
            onClick={onSend}
          >
            {busy ? "发送中" : "发送"}
          </button>
        </div>
      </div>
      <ImagePreviewOverlay src={previewImageSrc} onClose={() => setPreviewImageSrc(null)} />
    </section>
  );
}
