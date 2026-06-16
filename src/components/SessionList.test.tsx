import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SessionList } from "./SessionList";
import type { Session } from "../types";

const sessions: Session[] = [
  {
    id: "s1",
    project_id: "p1",
    title: "会话 A",
    model: "mock",
    thinking_enabled: true,
    thinking_effort: "high",
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
  {
    id: "s2",
    project_id: "p1",
    title: "会话 B",
    model: "mock",
    thinking_enabled: true,
    thinking_effort: "high",
    created_at: "2026-01-02T00:00:00Z",
    updated_at: "2026-01-02T00:00:00Z",
  },
];

describe("SessionList", () => {
  it("shows running and stopping indicators in sidebar", () => {
    render(
      <SessionList
        sessions={sessions}
        activeSessionId="s2"
        sessionRunStatuses={{ s1: "running", s2: "stopping" }}
        onSelectSession={vi.fn()}
        onDeleteSession={vi.fn()}
        onReorderSessions={vi.fn()}
      />,
    );

    expect(screen.getByLabelText("执行中")).toBeInTheDocument();
    expect(screen.getByLabelText("停止中")).toBeInTheDocument();
  });
});
