import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";
import { useComposerFocus } from "./useComposerFocus";

describe("useComposerFocus", () => {
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

  it("does not reset caret when import finishes", () => {
    const textarea = document.createElement("textarea");
    const textareaRef = { current: textarea };

    const { rerender } = renderHook(
      ({ composerDisabled, importing }) =>
        useComposerFocus({
          textareaRef,
          projectId: "p1",
          composerDisabled,
          importing,
          blockers: {},
        }),
      { initialProps: { composerDisabled: true, importing: true } },
    );

    focusSpy.mockClear();
    selectionSpy.mockClear();

    rerender({ composerDisabled: false, importing: false });
    flushRaf();

    expect(focusSpy).not.toHaveBeenCalled();
    expect(selectionSpy).not.toHaveBeenCalled();
  });

  it("refocuses when busy finishes and import is not involved", () => {
    const textarea = document.createElement("textarea");
    const textareaRef = { current: textarea };

    const { rerender } = renderHook(
      ({ composerDisabled }) =>
        useComposerFocus({
          textareaRef,
          projectId: "p1",
          composerDisabled,
          importing: false,
          blockers: {},
        }),
      { initialProps: { composerDisabled: true } },
    );

    focusSpy.mockClear();
    rerender({ composerDisabled: false });
    flushRaf();

    expect(focusSpy).toHaveBeenCalled();
    expect(selectionSpy).toHaveBeenCalledWith(0, 0);
  });
});
