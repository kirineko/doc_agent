import { describe, expect, it } from "vitest";
import {
  formatShortcut,
  isAddProjectShortcut,
  isZoomInShortcut,
  isZoomOutShortcut,
  isZoomResetShortcut,
} from "./keyboardShortcuts";

describe("keyboardShortcuts", () => {
  it("formats add-project shortcut", () => {
    expect(formatShortcut("o")).toMatch(/O$/);
  });

  it("detects add-project shortcut", () => {
    const macEvent = {
      metaKey: true,
      ctrlKey: false,
      shiftKey: false,
      altKey: false,
      key: "o",
    } as KeyboardEvent;
    const winEvent = {
      metaKey: false,
      ctrlKey: true,
      shiftKey: false,
      altKey: false,
      key: "O",
    } as KeyboardEvent;
    expect(isAddProjectShortcut(macEvent) || isAddProjectShortcut(winEvent)).toBe(true);
  });

  it("detects zoom in shortcut", () => {
    const macEvent = {
      metaKey: true,
      ctrlKey: false,
      shiftKey: false,
      altKey: false,
      isComposing: false,
      keyCode: 0,
      key: "=",
    } as KeyboardEvent;
    const winEvent = {
      metaKey: false,
      ctrlKey: true,
      shiftKey: false,
      altKey: false,
      isComposing: false,
      keyCode: 0,
      key: "=",
    } as KeyboardEvent;
    expect(isZoomInShortcut(macEvent) || isZoomInShortcut(winEvent)).toBe(true);
  });

  it("detects zoom out shortcut", () => {
    const winEvent = {
      metaKey: false,
      ctrlKey: true,
      shiftKey: false,
      altKey: false,
      isComposing: false,
      keyCode: 0,
      key: "-",
    } as KeyboardEvent;
    expect(isZoomOutShortcut(winEvent)).toBe(true);
  });

  it("detects zoom reset shortcut", () => {
    const macEvent = {
      metaKey: true,
      ctrlKey: false,
      shiftKey: false,
      altKey: false,
      isComposing: false,
      keyCode: 0,
      key: "0",
    } as KeyboardEvent;
    const winEvent = {
      metaKey: false,
      ctrlKey: true,
      shiftKey: false,
      altKey: false,
      isComposing: false,
      keyCode: 0,
      key: "0",
    } as KeyboardEvent;
    expect(isZoomResetShortcut(macEvent) || isZoomResetShortcut(winEvent)).toBe(true);
  });
});
