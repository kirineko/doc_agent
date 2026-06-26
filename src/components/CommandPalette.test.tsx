import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { CommandPalette } from "./CommandPalette";
import type { Project, Session } from "../types";

const projects: Project[] = [
  {
    id: "p1",
    name: "doc_test",
    root_path: "/tmp/doc_test",
    created_at: "2026-01-01",
  },
];

const sessions: Session[] = Array.from({ length: 12 }, (_, index) => ({
  id: `s${index}`,
  project_id: "p1",
  title: `Session ${index}`,
  model: "mock",
  thinking_enabled: false,
  thinking_effort: "high" as const,
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-01T00:00:00Z",
}));

describe("CommandPalette", () => {
  it("scrolls the highlighted item into view when navigating with arrow keys", () => {
    const scrollIntoView = vi.fn();
    vi.spyOn(HTMLElement.prototype, "scrollIntoView").mockImplementation(scrollIntoView);

    render(
      <CommandPalette
        open
        projects={projects}
        sessions={sessions}
        onClose={vi.fn()}
        onSelectItem={vi.fn()}
      />,
    );

    scrollIntoView.mockClear();
    fireEvent.keyDown(window, { key: "ArrowDown" });
    expect(scrollIntoView).toHaveBeenCalledWith({ block: "nearest" });

    scrollIntoView.mockClear();
    fireEvent.keyDown(window, { key: "ArrowDown" });
    expect(scrollIntoView).toHaveBeenCalledWith({ block: "nearest" });

    vi.restoreAllMocks();
  });

  it("keeps palette open when onSelectItem returns false", () => {
    const onClose = vi.fn();
    const onSelectItem = vi.fn(() => false);

    render(
      <CommandPalette
        open
        projects={[]}
        sessions={[]}
        onClose={onClose}
        onSelectItem={onSelectItem}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: /新建会话/ }));
    expect(onSelectItem).toHaveBeenCalled();
    expect(onClose).not.toHaveBeenCalled();
    expect(screen.getByRole("dialog", { name: "命令面板" })).toBeInTheDocument();
  });
});
