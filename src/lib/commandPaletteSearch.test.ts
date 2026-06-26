import { describe, expect, it } from "vitest";
import {
  buildCommandPaletteItems,
  parseSessionPaletteItemId,
  searchCommandPaletteItems,
} from "./commandPaletteSearch";
import type { Project, Session } from "../types";

const projects: Project[] = [
  {
    id: "p1",
    name: "doc_test",
    root_path: "/tmp/doc_test",
    created_at: "2026-01-01",
  },
  {
    id: "p2",
    name: "other_project",
    root_path: "/tmp/other",
    created_at: "2026-01-01",
  },
];

const sessions: Session[] = [
  {
    id: "s1",
    project_id: "p1",
    title: "排查 comments",
    model: "mock",
    thinking_enabled: true,
    thinking_effort: "high",
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
  {
    id: "s2",
    project_id: "p2",
    title: "Other session",
    model: "mock",
    thinking_enabled: false,
    thinking_effort: "high",
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
];

describe("commandPaletteSearch", () => {
  it("builds grouped items including actions and slash commands", () => {
    const items = buildCommandPaletteItems(projects, sessions);
    expect(items.some((item) => item.id === "action:new-session")).toBe(true);
    expect(items.some((item) => item.id === "project:p1")).toBe(true);
    expect(items.some((item) => item.id === "session:p1:s1")).toBe(true);
    expect(items.some((item) => item.id === "session:p2:s2")).toBe(true);
    expect(items.some((item) => item.id.startsWith("command:"))).toBe(true);
  });

  it("labels sessions with their project name", () => {
    const items = buildCommandPaletteItems(projects, sessions);
    const otherSession = items.find((item) => item.id === "session:p2:s2");
    expect(otherSession?.description).toBe("other_project");
  });

  it("parses cross-project session ids", () => {
    expect(parseSessionPaletteItemId("session:p2:s2")).toEqual({
      projectId: "p2",
      sessionId: "s2",
    });
    expect(parseSessionPaletteItemId("session:s1")).toBeNull();
  });

  it("filters sessions by fuzzy query", () => {
    const items = buildCommandPaletteItems(projects, sessions);
    const matches = searchCommandPaletteItems("comments", items);
    expect(matches.some((item) => item.id === "session:p1:s1")).toBe(true);
  });
});
