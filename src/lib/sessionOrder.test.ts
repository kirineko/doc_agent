import { beforeEach, describe, expect, it } from "vitest";
import {
  SESSION_ORDER_STORAGE_KEY,
  applySessionOrder,
  displaySessionsForProject,
  hasManualOrder,
  moveSessionInList,
  prependSessionToOrder,
  readProjectOrder,
  removeSessionFromOrder,
  writeProjectOrder,
} from "./sessionOrder";
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

describe("sessionOrder", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("lazy activation: no storage entry means auto order", () => {
    const sessions = [session("s1", "2026-01-02"), session("s2", "2026-01-01")];
    expect(hasManualOrder("p1")).toBe(false);
    expect(readProjectOrder("p1")).toBeNull();
    expect(displaySessionsForProject(sessions, "p1")).toEqual(sessions);
  });

  it("writeProjectOrder activates manual order for project", () => {
    writeProjectOrder("p1", ["s2", "s1"]);
    expect(hasManualOrder("p1")).toBe(true);
    expect(readProjectOrder("p1")).toEqual(["s2", "s1"]);
  });

  it("applySessionOrder filters orphan ids and prepends unknown sessions", () => {
    const sessions = [
      session("s1", "2026-01-03"),
      session("s2", "2026-01-02"),
      session("s3", "2026-01-01"),
    ];
    const ordered = applySessionOrder(sessions, ["missing", "s2", "s1"]);
    expect(ordered.map((s) => s.id)).toEqual(["s3", "s2", "s1"]);
  });

  it("prependSessionToOrder only applies in manual mode", () => {
    prependSessionToOrder("p1", "s-new");
    expect(readProjectOrder("p1")).toBeNull();

    writeProjectOrder("p1", ["s2", "s1"]);
    prependSessionToOrder("p1", "s-new");
    expect(readProjectOrder("p1")).toEqual(["s-new", "s2", "s1"]);
  });

  it("moveSessionInList preserves sessions added after drag started", () => {
    const sessions = [
      session("d-new", "2026-01-04"),
      session("a", "2026-01-03"),
      session("b", "2026-01-02"),
      session("c", "2026-01-01"),
    ];
    const next = moveSessionInList(sessions, "b", "c");
    expect(next?.map((s) => s.id)).toEqual(["d-new", "a", "c", "b"]);
  });

  it("moveSessionInList returns null for unknown ids", () => {
    const sessions = [session("a", "2026-01-01")];
    expect(moveSessionInList(sessions, "missing", "a")).toBeNull();
  });

  it("removeSessionFromOrder only applies in manual mode", () => {
    removeSessionFromOrder("p1", "s1");
    expect(readProjectOrder("p1")).toBeNull();

    writeProjectOrder("p1", ["s2", "s1"]);
    removeSessionFromOrder("p1", "s1");
    expect(readProjectOrder("p1")).toEqual(["s2"]);
  });

  it("removeSessionFromOrder clears storage when last session id removed", () => {
    writeProjectOrder("p1", ["s1"]);
    removeSessionFromOrder("p1", "s1");
    expect(hasManualOrder("p1")).toBe(false);
    expect(readProjectOrder("p1")).toBeNull();
    expect(localStorage.getItem(SESSION_ORDER_STORAGE_KEY)).toBe("{}");
  });

  it("ignores invalid localStorage payload", () => {
    localStorage.setItem(SESSION_ORDER_STORAGE_KEY, "not-json");
    expect(readProjectOrder("p1")).toBeNull();

    localStorage.setItem(SESSION_ORDER_STORAGE_KEY, JSON.stringify(["bad"]));
    expect(readProjectOrder("p1")).toBeNull();
  });
});
