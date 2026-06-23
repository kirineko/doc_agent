import type { KeyboardEvent } from "react";
import type { FileMentionMatch } from "./mentionFiles";
import { deleteMentionBeforeCursor } from "./mention";
import type { MentionState } from "./mention";
import { deletePlaceholderAtCursor, deletePlaceholderBeforeCursor } from "./promptPlaceholder";
import type { SlashCommandMatch } from "./slashFuzzy";

export interface ChatInputKeyDownContext {
  input: string;
  cursor: number;
  inputDisabled: boolean;
  canSend: boolean;
  mention: MentionState | null;
  mentionDismissed: boolean;
  mentionDisplayMatches: FileMentionMatch[];
  mentionIndex: number;
  showSlashPopup: boolean;
  slashMatches: SlashCommandMatch[];
  slashIndex: number;
  onInputChange: (text: string) => void;
  setCursor: (value: number) => void;
  focusTextareaAt: (start: number, end?: number) => void;
  setMentionDismissed: (value: boolean) => void;
  setMentionIndex: (value: number | ((index: number) => number)) => void;
  pickMention: (path: string) => void;
  browseMentionDirectory: (path: string) => void;
  setSlashDismissed: (value: boolean) => void;
  setSlashIndex: (value: number | ((index: number) => number)) => void;
  pickSlash: (commandId: string) => void;
  onSend: () => void;
}

export function handleChatInputKeyDown(
  event: KeyboardEvent<HTMLTextAreaElement>,
  ctx: ChatInputKeyDownContext,
): void {
  // IME 组合中（中文/日文输入法选词）不拦截任何键，交给浏览器原生处理，
  // 否则 Enter 确认候选词会误触发发送、Backspace 误删占位符等。
  if (event.nativeEvent.isComposing || event.keyCode === 229) return;

  if (event.key === "Backspace" && !event.shiftKey) {
    const deletedPh = deletePlaceholderBeforeCursor(ctx.input, ctx.cursor);
    if (deletedPh) {
      event.preventDefault();
      ctx.onInputChange(deletedPh.text);
      ctx.setCursor(deletedPh.cursor);
      ctx.focusTextareaAt(deletedPh.cursor);
      return;
    }
    const deleted = deleteMentionBeforeCursor(ctx.input, ctx.cursor);
    if (deleted) {
      event.preventDefault();
      ctx.onInputChange(deleted.text);
      ctx.setCursor(deleted.cursor);
      ctx.focusTextareaAt(deleted.cursor);
      return;
    }
  }
  if (event.key === "Delete" && !event.shiftKey) {
    const deletedPh = deletePlaceholderAtCursor(ctx.input, ctx.cursor);
    if (deletedPh) {
      event.preventDefault();
      ctx.onInputChange(deletedPh.text);
      ctx.setCursor(deletedPh.cursor);
      ctx.focusTextareaAt(deletedPh.cursor);
      return;
    }
  }
  if (ctx.mention && !ctx.mentionDismissed) {
    if (event.key === "Escape") {
      event.preventDefault();
      ctx.setMentionDismissed(true);
      return;
    }
    if (ctx.mentionDisplayMatches.length > 0) {
      const count = ctx.mentionDisplayMatches.length;
      if (event.key === "ArrowDown") {
        event.preventDefault();
        ctx.setMentionIndex((index) => (index + 1) % count);
        return;
      }
      if (event.key === "ArrowUp") {
        event.preventDefault();
        ctx.setMentionIndex((index) => (index - 1 + count) % count);
        return;
      }
      if (event.key === "Enter") {
        event.preventDefault();
        ctx.pickMention(
          ctx.mentionDisplayMatches[ctx.mentionIndex]?.item ??
            ctx.mentionDisplayMatches[0]!.item,
        );
        return;
      }
      if (event.key === "Tab") {
        event.preventDefault();
        const selected =
          ctx.mentionDisplayMatches[ctx.mentionIndex] ?? ctx.mentionDisplayMatches[0]!;
        if (selected.isDir) {
          ctx.browseMentionDirectory(selected.item);
        } else {
          ctx.pickMention(selected.item);
        }
        return;
      }
    } else if (event.key === "Enter" || event.key === "Tab") {
      event.preventDefault();
      return;
    }
  }
  if (ctx.showSlashPopup) {
    if (event.key === "Escape") {
      event.preventDefault();
      ctx.setSlashDismissed(true);
      return;
    }
    if (ctx.slashMatches.length > 0) {
      const count = ctx.slashMatches.length;
      if (event.key === "ArrowDown") {
        event.preventDefault();
        ctx.setSlashIndex((index) => (index + 1) % count);
        return;
      }
      if (event.key === "ArrowUp") {
        event.preventDefault();
        ctx.setSlashIndex((index) => (index - 1 + count) % count);
        return;
      }
      if (event.key === "Enter" || event.key === "Tab") {
        event.preventDefault();
        const match = ctx.slashMatches[ctx.slashIndex] ?? ctx.slashMatches[0]!;
        ctx.pickSlash(match.command.id);
        return;
      }
    } else if (event.key === "Enter" || event.key === "Tab") {
      event.preventDefault();
      return;
    }
  }
  if (event.key === "Enter" && !event.shiftKey && !ctx.inputDisabled && ctx.canSend) {
    event.preventDefault();
    ctx.onSend();
  }
}
