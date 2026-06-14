import { describe, expect, it } from "vitest";
import { mostRecentSessionId, shouldApplyProjectSelection } from "./projectSession";
import type { Session } from "../types";

function session(id: string, updatedAt: string): Session {
  return {
    id,
    project_id: "p1",
    title: "t",
    model: "deepseek-v4-flash",
    thinking_enabled: true,
    thinking_effort: "high",
    created_at: updatedAt,
    updated_at: updatedAt,
  };
}

describe("projectSession", () => {
  it("picks session with latest updated_at regardless of array order", () => {
    const sessions = [session("s-old", "2026-01-01"), session("s-new", "2026-01-02")];
    expect(mostRecentSessionId(sessions)).toBe("s-new");
  });

  it("returns undefined for empty session list", () => {
    expect(mostRecentSessionId([])).toBeUndefined();
  });

  it("detects stale project selection results", () => {
    expect(shouldApplyProjectSelection("a", "a")).toBe(true);
    expect(shouldApplyProjectSelection("a", "b")).toBe(false);
  });
});
