import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ProjectSessionTree } from "./ProjectSessionTree";
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
    name: "other",
    root_path: "/tmp/other",
    created_at: "2026-01-01",
  },
];

const sessions: Session[] = [
  {
    id: "s1",
    project_id: "p1",
    title: "Review change",
    model: "mock",
    thinking_enabled: true,
    thinking_effort: "high",
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
];

describe("ProjectSessionTree", () => {
  it("renders action rail without sidebar model trigger", () => {
    render(
      <ProjectSessionTree
        projects={projects}
        sessions={sessions}
        activeProjectId="p1"
        activeSessionId="s1"
        onProjectsChange={vi.fn()}
        onSelectProject={vi.fn()}
        onSelectSession={vi.fn()}
        onCreateSession={vi.fn()}
        onDeleteSession={vi.fn()}
        onReorderSessions={vi.fn()}
        onOpenSearch={vi.fn()}
        onPromptAddProject={vi.fn()}
        onAddProject={vi.fn()}
      />,
    );

    expect(screen.getByText("添加项目目录")).toBeInTheDocument();
    expect(screen.getByText("新建会话")).toBeInTheDocument();
    expect(screen.getByText("Review change")).toBeInTheDocument();
    expect(screen.queryByText("＋ 添加项目目录")).not.toBeInTheDocument();
    expect(screen.queryByText("模型")).not.toBeInTheDocument();
    expect(document.getElementById("sidebar-model-trigger")).toBeNull();
  });

  it("shows ghost add project instead of primary full-width button", () => {
    render(
      <ProjectSessionTree
        projects={projects}
        sessions={sessions}
        activeProjectId="p1"
        onProjectsChange={vi.fn()}
        onSelectProject={vi.fn()}
        onSelectSession={vi.fn()}
        onCreateSession={vi.fn()}
        onDeleteSession={vi.fn()}
        onReorderSessions={vi.fn()}
        onOpenSearch={vi.fn()}
        onPromptAddProject={vi.fn()}
        onAddProject={vi.fn()}
      />,
    );

    expect(screen.queryByText("选择目录创建项目")).not.toBeInTheDocument();
  });

  it("highlights add project action when highlightProject is set", () => {
    render(
      <ProjectSessionTree
        projects={[]}
        sessions={[]}
        highlightProject
        onProjectsChange={vi.fn()}
        onSelectProject={vi.fn()}
        onSelectSession={vi.fn()}
        onCreateSession={vi.fn()}
        onDeleteSession={vi.fn()}
        onReorderSessions={vi.fn()}
        onOpenSearch={vi.fn()}
        onPromptAddProject={vi.fn()}
        onAddProject={vi.fn()}
      />,
    );

    expect(document.getElementById("sidebar-add-project")).toHaveClass("ring-amber-600/60");
    expect(document.getElementById("sidebar-projects")).toHaveClass("ring-amber-600/60");
  });

  it("toggles active project sessions when clicking project header", () => {
    render(
      <ProjectSessionTree
        projects={projects}
        sessions={sessions}
        activeProjectId="p1"
        activeSessionId="s1"
        onProjectsChange={vi.fn()}
        onSelectProject={vi.fn()}
        onSelectSession={vi.fn()}
        onCreateSession={vi.fn()}
        onDeleteSession={vi.fn()}
        onReorderSessions={vi.fn()}
        onOpenSearch={vi.fn()}
        onPromptAddProject={vi.fn()}
        onAddProject={vi.fn()}
      />,
    );

    expect(screen.getByText("Review change")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "doc_test" }));
    expect(screen.queryByText("Review change")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "doc_test" }));
    expect(screen.getByText("Review change")).toBeInTheDocument();
  });

  it("expands project before creating a new session", () => {
    const onCreateSession = vi.fn();
    render(
      <ProjectSessionTree
        projects={projects}
        sessions={sessions}
        activeProjectId="p1"
        onProjectsChange={vi.fn()}
        onSelectProject={vi.fn()}
        onSelectSession={vi.fn()}
        onCreateSession={onCreateSession}
        onDeleteSession={vi.fn()}
        onReorderSessions={vi.fn()}
        onOpenSearch={vi.fn()}
        onPromptAddProject={vi.fn()}
        onAddProject={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "doc_test" }));
    expect(screen.queryByText("Review change")).not.toBeInTheDocument();

    fireEvent.click(screen.getByText("新建会话"));
    expect(screen.getByText("Review change")).toBeInTheDocument();
    expect(onCreateSession).toHaveBeenCalledTimes(1);
  });

  it("expands active project when draft send creates the first session", () => {
    const { rerender } = render(
      <ProjectSessionTree
        projects={projects}
        sessions={[]}
        activeProjectId="p1"
        onProjectsChange={vi.fn()}
        onSelectProject={vi.fn()}
        onSelectSession={vi.fn()}
        onCreateSession={vi.fn()}
        onDeleteSession={vi.fn()}
        onReorderSessions={vi.fn()}
        onOpenSearch={vi.fn()}
        onPromptAddProject={vi.fn()}
        onAddProject={vi.fn()}
      />,
    );

    expect(screen.getByText("暂无会话")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "doc_test" }));
    expect(screen.queryByText("暂无会话")).not.toBeInTheDocument();

    rerender(
      <ProjectSessionTree
        projects={projects}
        sessions={[
          {
            id: "s-new",
            project_id: "p1",
            title: "新会话",
            model: "mock",
            thinking_enabled: true,
            thinking_effort: "high",
            created_at: "2026-01-02T00:00:00Z",
            updated_at: "2026-01-02T00:00:00Z",
          },
        ]}
        activeProjectId="p1"
        activeSessionId="s-new"
        onProjectsChange={vi.fn()}
        onSelectProject={vi.fn()}
        onSelectSession={vi.fn()}
        onCreateSession={vi.fn()}
        onDeleteSession={vi.fn()}
        onReorderSessions={vi.fn()}
        onOpenSearch={vi.fn()}
        onPromptAddProject={vi.fn()}
        onAddProject={vi.fn()}
      />,
    );

    expect(screen.getByText("新会话")).toBeInTheDocument();
  });

  it("prompts add project when creating session without active project", () => {
    const onPromptAddProject = vi.fn();
    render(
      <ProjectSessionTree
        projects={[]}
        sessions={[]}
        onProjectsChange={vi.fn()}
        onSelectProject={vi.fn()}
        onSelectSession={vi.fn()}
        onCreateSession={vi.fn()}
        onDeleteSession={vi.fn()}
        onReorderSessions={vi.fn()}
        onOpenSearch={vi.fn()}
        onPromptAddProject={onPromptAddProject}
        onAddProject={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByText("新建会话"));
    expect(onPromptAddProject).toHaveBeenCalledTimes(1);
  });

  it("creates a session in the target project from another project row", async () => {
    const onSelectProject = vi.fn();
    const onCreateSession = vi.fn().mockResolvedValue(undefined);
    render(
      <ProjectSessionTree
        projects={projects}
        sessions={sessions}
        activeProjectId="p1"
        onProjectsChange={vi.fn()}
        onSelectProject={onSelectProject}
        onSelectSession={vi.fn()}
        onCreateSession={onCreateSession}
        onDeleteSession={vi.fn()}
        onReorderSessions={vi.fn()}
        onOpenSearch={vi.fn()}
        onPromptAddProject={vi.fn()}
        onAddProject={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "在 other 新建会话" }));
    await vi.waitFor(() => {
      expect(onCreateSession).toHaveBeenCalledWith("p2");
    });
    expect(onSelectProject).not.toHaveBeenCalled();
  });
});
