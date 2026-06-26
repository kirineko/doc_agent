import { describe, expect, it } from "vitest";
import { formatShortcut, isAddProjectShortcut } from "./keyboardShortcuts";

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
});
