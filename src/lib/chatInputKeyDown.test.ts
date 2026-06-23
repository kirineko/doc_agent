import { describe, expect, it, vi } from "vitest";
import type { KeyboardEvent } from "react";
import { handleChatInputKeyDown, type ChatInputKeyDownContext } from "./chatInputKeyDown";

function makeEvent(
  key: string,
  overrides: { isComposing?: boolean; keyCode?: number; shiftKey?: boolean } = {},
) {
  return {
    key,
    shiftKey: overrides.shiftKey ?? false,
    keyCode: overrides.keyCode ?? 0,
    nativeEvent: { isComposing: overrides.isComposing ?? false } as KeyboardEvent["nativeEvent"],
    preventDefault: vi.fn(),
  } as unknown as KeyboardEvent<HTMLTextAreaElement>;
}

function makeCtx(overrides: Partial<ChatInputKeyDownContext> = {}): ChatInputKeyDownContext {
  return {
    input: "",
    cursor: 0,
    inputDisabled: false,
    canSend: true,
    mention: null,
    mentionDismissed: false,
    mentionDisplayMatches: [],
    mentionIndex: 0,
    showSlashPopup: false,
    slashMatches: [],
    slashIndex: 0,
    onInputChange: vi.fn(),
    setCursor: vi.fn(),
    focusTextareaAt: vi.fn(),
    setMentionDismissed: vi.fn(),
    setMentionIndex: vi.fn(),
    pickMention: vi.fn(),
    browseMentionDirectory: vi.fn(),
    setSlashDismissed: vi.fn(),
    setSlashIndex: vi.fn(),
    pickSlash: vi.fn(),
    onSend: vi.fn(),
    ...overrides,
  };
}

describe("handleChatInputKeyDown IME guard", () => {
  it("does not send when Enter is pressed during IME composition", () => {
    const onSend = vi.fn();
    const event = makeEvent("Enter", { isComposing: true });

    handleChatInputKeyDown(event, makeCtx({ onSend }));

    expect(event.preventDefault).not.toHaveBeenCalled();
    expect(onSend).not.toHaveBeenCalled();
  });

  it("does not send when Enter has IME keyCode 229", () => {
    const onSend = vi.fn();
    const event = makeEvent("Enter", { keyCode: 229 });

    handleChatInputKeyDown(event, makeCtx({ onSend }));

    expect(onSend).not.toHaveBeenCalled();
  });

  it("sends on Enter when not composing", () => {
    const onSend = vi.fn();
    const event = makeEvent("Enter");

    handleChatInputKeyDown(event, makeCtx({ onSend }));

    expect(event.preventDefault).toHaveBeenCalled();
    expect(onSend).toHaveBeenCalled();
  });
});
