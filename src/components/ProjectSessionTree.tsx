import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from "react";
import { formatShortcut, isMacPlatform } from "../lib/keyboardShortcuts";
import type { SessionRunStatus } from "../lib/sessionRunState";
import type { Project, Session } from "../types";
import {
  ChevronDownIcon,
  ChevronRightIcon,
  FolderIcon,
  FolderOpenIcon,
} from "./PanelIcons";
import { SessionList } from "./SessionList";

interface ProjectSessionTreeProps {
  projects: Project[];
  sessions: Session[];
  activeProjectId?: string;
  activeSessionId?: string;
  sessionRunStatuses?: Record<string, SessionRunStatus>;
  highlightProject?: boolean;
  onProjectsChange: (projects: Project[]) => void;
  onSelectProject: (projectId: string | undefined) => void | Promise<void>;
  onSelectSession: (sessionId: string | undefined) => void;
  onCreateSession: (projectId?: string) => void | Promise<void>;
  onDeleteSession: (sessionId: string) => void | Promise<void>;
  onReorderSessions: (activeId: string, overId: string) => void;
  onOpenSearch: () => void;
  onPromptAddProject: () => void;
  onAddProject: () => void | Promise<void>;
}

const SIDEBAR_HIGHLIGHT_CLASS = "rounded-md bg-amber-500/5 ring-1 ring-amber-600/60";

export function ProjectSessionTree({
  projects,
  sessions,
  activeProjectId,
  activeSessionId,
  sessionRunStatuses,
  highlightProject,
  onProjectsChange,
  onSelectProject,
  onSelectSession,
  onCreateSession,
  onDeleteSession,
  onReorderSessions,
  onOpenSearch,
  onPromptAddProject,
  onAddProject,
}: ProjectSessionTreeProps) {
  const [openMenuProjectId, setOpenMenuProjectId] = useState<string | null>(null);
  const [collapsedProjects, setCollapsedProjects] = useState<Set<string>>(() => new Set());
  const menuRef = useRef<HTMLDivElement>(null);

  const expandProject = useCallback((projectId: string) => {
    setCollapsedProjects((current) => {
      if (!current.has(projectId)) return current;
      const next = new Set(current);
      next.delete(projectId);
      return next;
    });
  }, []);

  useEffect(() => {
    if (!activeProjectId) return;
    expandProject(activeProjectId);
  }, [activeProjectId, activeSessionId, expandProject]);

  useEffect(() => {
    if (!openMenuProjectId) return;
    function handlePointerDown(event: MouseEvent) {
      if (menuRef.current?.contains(event.target as Node)) return;
      setOpenMenuProjectId(null);
    }
    window.addEventListener("mousedown", handlePointerDown);
    return () => window.removeEventListener("mousedown", handlePointerDown);
  }, [openMenuProjectId]);

  async function hideProject(projectId: string) {
    try {
      await invoke("hide_project", { projectId });
      const next = projects.filter((item) => item.id !== projectId);
      onProjectsChange(next);
      if (activeProjectId === projectId) {
        await onSelectProject(next[0]?.id);
      }
    } catch (error) {
      console.error(error);
    } finally {
      setOpenMenuProjectId(null);
    }
  }

  async function openProjectRoot(projectId: string) {
    try {
      await invoke("open_project_root", { projectId });
    } catch (error) {
      console.error(error);
    } finally {
      setOpenMenuProjectId(null);
    }
  }

  async function handleCreateSession(projectId?: string) {
    const targetId = projectId ?? activeProjectId;
    if (!targetId) {
      onPromptAddProject();
      return;
    }
    expandProject(targetId);
    await onCreateSession(targetId);
  }

  function toggleProjectExpanded(projectId: string) {
    setCollapsedProjects((current) => {
      const next = new Set(current);
      if (next.has(projectId)) {
        next.delete(projectId);
      } else {
        next.add(projectId);
      }
      return next;
    });
  }

  async function handleProjectHeaderClick(projectId: string) {
    if (projectId === activeProjectId) {
      toggleProjectExpanded(projectId);
      return;
    }
    await onSelectProject(projectId);
  }

  const openInFolderLabel = isMacPlatform() ? "在 Finder 中打开" : "在文件夹中打开";

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-2">
      <div className="flex shrink-0 flex-col gap-1">
        <button
          id="sidebar-add-project"
          type="button"
          className={`nav-action flex w-full items-center justify-between px-2 py-1.5 text-left text-xs text-fg hover:bg-hover ${
            highlightProject ? SIDEBAR_HIGHLIGHT_CLASS : "rounded-md"
          }`}
          onClick={() => void onAddProject()}
        >
          <span>添加项目目录</span>
          <span className="text-[10px] text-fg-muted">{formatShortcut("o")}</span>
        </button>
        <button
          type="button"
          className="nav-action flex w-full items-center justify-between rounded-md px-2 py-1.5 text-left text-xs text-fg hover:bg-hover"
          onClick={() => void handleCreateSession()}
        >
          <span>新建会话</span>
          <span className="text-[10px] text-fg-muted">{formatShortcut("n")}</span>
        </button>
        <button
          type="button"
          className="nav-action flex w-full items-center justify-between rounded-md px-2 py-1.5 text-left text-xs text-fg hover:bg-hover"
          onClick={() => onOpenSearch()}
        >
          <span>搜索</span>
          <span className="text-[10px] text-fg-muted">{formatShortcut("k")}</span>
        </button>
      </div>

      <div
        id="sidebar-projects"
        className={`min-h-0 flex-1 overflow-y-auto ${highlightProject ? SIDEBAR_HIGHLIGHT_CLASS : ""}`}
      >
        <div className="space-y-0.5">
          {projects.map((project) => {
            const active = project.id === activeProjectId;
            const expanded = active && !collapsedProjects.has(project.id);
            return (
              <div key={project.id}>
                <div
                  className={`group relative flex items-center rounded-md text-xs ${
                    active ? "nav-project-active bg-hover" : "hover:bg-hover"
                  }`}
                >
                  <button
                    type="button"
                    className="flex min-w-0 flex-1 items-center rounded-md py-0.5 pl-0.5 pr-1 text-left"
                    title={project.root_path}
                    aria-expanded={active ? expanded : false}
                    aria-label={project.name}
                    onClick={() => void handleProjectHeaderClick(project.id)}
                  >
                    <span className="inline-flex shrink-0 items-center justify-center p-1 text-fg-muted">
                      {expanded ? (
                        <ChevronDownIcon className="h-3 w-3" />
                      ) : (
                        <ChevronRightIcon className="h-3 w-3" />
                      )}
                    </span>
                    <span className="inline-flex shrink-0 items-center justify-center px-0.5 text-fg-muted">
                      {expanded ? (
                        <FolderOpenIcon className="h-3.5 w-3.5" />
                      ) : (
                        <FolderIcon className="h-3.5 w-3.5" />
                      )}
                    </span>
                    <span className="min-w-0 flex-1 truncate py-1 font-medium">{project.name}</span>
                  </button>
                  <button
                    type="button"
                    className="shrink-0 rounded px-1.5 py-1 text-fg-muted opacity-0 transition hover:text-fg group-hover:opacity-100"
                    title="在此项目新建会话"
                    aria-label={`在 ${project.name} 新建会话`}
                    onClick={() => void handleCreateSession(project.id)}
                  >
                    +
                  </button>
                  <div className="relative shrink-0">
                    <button
                      type="button"
                      className="rounded px-1.5 py-1 text-fg-muted opacity-0 transition hover:text-fg group-hover:opacity-100"
                      aria-label={`${project.name} 菜单`}
                      aria-expanded={openMenuProjectId === project.id}
                      onClick={() =>
                        setOpenMenuProjectId((current) =>
                          current === project.id ? null : project.id,
                        )
                      }
                    >
                      ···
                    </button>
                    {openMenuProjectId === project.id && (
                      <div
                        ref={menuRef}
                        className="absolute right-0 top-full z-20 mt-1 min-w-[10rem] rounded-md border border-border-subtle bg-elevated py-1 shadow-lg"
                      >
                        <button
                          type="button"
                          className="block w-full px-3 py-1.5 text-left text-xs hover:bg-hover"
                          onClick={() => void openProjectRoot(project.id)}
                        >
                          {openInFolderLabel}
                        </button>
                        <button
                          type="button"
                          className="block w-full px-3 py-1.5 text-left text-xs text-rose-500 hover:bg-hover"
                          onClick={() => void hideProject(project.id)}
                        >
                          从列表移除
                        </button>
                      </div>
                    )}
                  </div>
                </div>
                {expanded && (
                  <div className="ml-2 border-l border-border-subtle pl-1.5 pt-0.5">
                    {sessions.length === 0 ? (
                      <div className="px-2 py-1.5 text-[11px] text-fg-muted">暂无会话</div>
                    ) : (
                      <SessionList
                        sessions={sessions}
                        activeSessionId={activeSessionId}
                        sessionRunStatuses={sessionRunStatuses}
                        onSelectSession={onSelectSession}
                        onDeleteSession={(sessionId) => void onDeleteSession(sessionId)}
                        onReorderSessions={onReorderSessions}
                        variant="tree"
                      />
                    )}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
