import { describe, expect, it } from "vitest";
import {
  countChatMessages,
  isStaleSessionResult,
  shouldDiscardFollowup,
  shouldRunStarter,
} from "./suggestions";
import type { Message } from "../types";

describe("suggestions helpers", () => {
  it("skips starter without deepseek key", () => {
    expect(shouldRunStarter(false, 0, false)).toBe(false);
  });

  it("runs starter for empty session with key", () => {
    expect(shouldRunStarter(true, 0, false)).toBe(true);
  });

  it("does not run starter when already initializing", () => {
    expect(shouldRunStarter(true, 0, true)).toBe(false);
  });

  it("counts only user and assistant messages", () => {
    const messages = [
      { role: "user" },
      { role: "assistant" },
      { role: "tool" },
    ] as Message[];
    expect(countChatMessages(messages)).toBe(2);
  });

  it("discards stale session results", () => {
    expect(isStaleSessionResult("s1", "s2")).toBe(true);
    expect(isStaleSessionResult("s1", "s1")).toBe(false);
  });

  it("discards followup when user sent new message", () => {
    expect(shouldDiscardFollowup(2, 4)).toBe(true);
    expect(shouldDiscardFollowup(4, 4)).toBe(false);
  });
});
