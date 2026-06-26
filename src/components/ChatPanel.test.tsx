import { render, screen } from "@testing-library/react";
import type { ComponentProps } from "react";
import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";
import { ChatPanel } from "./ChatPanel";

function baseProps(overrides: Partial<ComponentProps<typeof ChatPanel>> = {}) {
  return {
    messages: [],
    streamingReasoning: "",
    streamingContent: "",
    input: "",
    busy: false,
    pendingAttachments: [],
    onInputChange: vi.fn(),
    onSend: vi.fn(),
    onPasteImage: vi.fn(),
    onRemoveAttachment: vi.fn(),
    projectId: "p1",
    ...overrides,
  };
}

describe("ChatPanel composer focus", () => {
  let rafCallbacks: FrameRequestCallback[];
  let focusSpy: ReturnType<typeof vi.spyOn>;
  let selectionSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    rafCallbacks = [];
    vi.spyOn(window, "requestAnimationFrame").mockImplementation((cb) => {
      rafCallbacks.push(cb);
      return rafCallbacks.length;
    });
    focusSpy = vi.spyOn(HTMLTextAreaElement.prototype, "focus").mockImplementation(() => {});
    selectionSpy = vi
      .spyOn(HTMLTextAreaElement.prototype, "setSelectionRange")
      .mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  function flushRaf() {
    const pending = [...rafCallbacks];
    rafCallbacks = [];
    pending.forEach((cb) => cb(0));
  }

  it("refocuses textarea when composer becomes enabled after busy", () => {
    const { rerender } = render(<ChatPanel {...baseProps({ busy: true })} />);
    screen.getByRole("textbox").focus();

    rerender(<ChatPanel {...baseProps({ busy: false })} />);
    flushRaf();

    expect(focusSpy).toHaveBeenCalled();
    expect(selectionSpy).toHaveBeenCalledWith(0, 0);
  });

  it("refocuses even when focus moved outside composer before rAF runs", () => {
    const external = document.createElement("button");
    document.body.appendChild(external);

    const { rerender } = render(<ChatPanel {...baseProps({ busy: true })} />);
    focusSpy.mockClear();

    rerender(<ChatPanel {...baseProps({ busy: false })} />);
    external.focus();
    flushRaf();

    expect(focusSpy).toHaveBeenCalled();

    external.remove();
  });

  it("does not refocus when settings drawer blocker is active", () => {
    const { rerender } = render(
      <ChatPanel
        {...baseProps({
          busy: true,
          composerFocusBlockers: { settingsOpen: true },
        })}
      />,
    );

    rerender(
      <ChatPanel
        {...baseProps({
          busy: false,
          composerFocusBlockers: { settingsOpen: true },
        })}
      />,
    );
    flushRaf();

    expect(focusSpy).not.toHaveBeenCalled();
  });

  it("refocuses when sessionId changes", () => {
    const { rerender } = render(<ChatPanel {...baseProps({ sessionId: "s1" })} />);
    focusSpy.mockClear();

    rerender(<ChatPanel {...baseProps({ sessionId: "s2" })} />);
    flushRaf();

    expect(focusSpy).toHaveBeenCalled();
    expect(selectionSpy).toHaveBeenCalledWith(0, 0);
  });

  it("refocuses when first session becomes active", () => {
    const { rerender } = render(<ChatPanel {...baseProps()} />);
    focusSpy.mockClear();

    rerender(<ChatPanel {...baseProps({ sessionId: "s1" })} />);
    flushRaf();

    expect(focusSpy).toHaveBeenCalled();
  });

  it("does not refocus on initial mount with sessionId", () => {
    render(<ChatPanel {...baseProps({ sessionId: "s1" })} />);
    flushRaf();

    expect(focusSpy).not.toHaveBeenCalled();
  });
});

describe("ChatPanel empty starter row", () => {
  it("shows direct-input hint on the same row as init capsule", () => {
    render(
      <ChatPanel
        {...baseProps({
          showInitCapsule: true,
          onInitStarter: vi.fn(),
        })}
      />,
    );

    expect(screen.getByText("根据文档生成推荐问")).toBeInTheDocument();
    expect(screen.getByText("或直接输入开始对话")).toBeInTheDocument();
  });

  it("hides direct-input hint after starter suggestions are generated", () => {
    render(
      <ChatPanel
        {...baseProps({
          showInitCapsule: false,
          starterSuggestions: ["Summarize the report"],
        })}
      />,
    );

    expect(screen.queryByText("或直接输入开始对话")).not.toBeInTheDocument();
  });

  it("hides direct-input hint while generating starter suggestions", () => {
    render(
      <ChatPanel
        {...baseProps({
          showInitCapsule: true,
          initializing: true,
          onInitStarter: vi.fn(),
        })}
      />,
    );

    expect(screen.getByText("正在分析项目文档…")).toBeInTheDocument();
    expect(screen.queryByText("或直接输入开始对话")).not.toBeInTheDocument();
  });
});
