import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { pathBasename } from "../lib/pathUtils";
import { type SessionConfig } from "../lib/sessionConfig";
import { Project, Session } from "../types";
import { SessionList } from "./SessionList";
import { WebSearchSection } from "./WebSearchSection";

interface SidebarProps {
  projects: Project[];
  sessions: Session[];
  activeProjectId?: string;
  activeSessionId?: string;
  apiKeyStatus: Record<string, boolean>;
  pendingSessionConfig: SessionConfig;
  modelLocked: boolean;
  highlightProject?: boolean;
  highlightApiKeyProvider?: string;
  tavilyEnabled: boolean;
  onProjectsChange: (projects: Project[]) => void;
  onSelectProject: (projectId: string | undefined) => void | Promise<void>;
  onSelectSession: (sessionId: string | undefined) => void;
  onCreateSession: () => void | Promise<void>;
  onDeleteSession: (sessionId: string) => void | Promise<void>;
  onReorderSessions: (activeId: string, overId: string) => void;
  onPendingSessionConfigChange: (patch: Partial<SessionConfig>) => void;
  onSessionUpdated: (session: Session) => void;
  onApiKeyStatusChange: (provider: string, has: boolean) => void;
  onTavilyStatusChange: (has: boolean) => void;
  onOpenModelSettings: () => void;
  modelSummary: string;
}

export function Sidebar({
  projects,
  sessions,
  activeProjectId,
  activeSessionId,
  modelLocked,
  highlightProject,
  tavilyEnabled,
  onProjectsChange,
  onSelectProject,
  onSelectSession,
  onCreateSession,
  onDeleteSession,
  onReorderSessions,
  onTavilyStatusChange,
  onOpenModelSettings,
  modelSummary,
}: SidebarProps) {
  async function pickProject() {
    const selected = await open({ directory: true, multiple: false });
    if (!selected || Array.isArray(selected)) return;
    const name = pathBasename(selected) || "project";
    const project = await invoke<Project>("create_project", {
      req: { name, root_path: selected },
    });
    onProjectsChange([project, ...projects]);
    await onSelectProject(project.id);
  }

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
    }
  }

  return (
    <aside className="panel flex h-full w-72 shrink-0 flex-col gap-2.5 p-3">
      <div
        id="sidebar-projects"
        className={`shrink-0 rounded-md ${highlightProject ? "ring-1 ring-amber-600/60" : ""}`}
      >
        <div className="mb-1 text-[11px] uppercase tracking-[0.16em] text-fg-secondary">项目</div>
        <button
          className="btn-primary mb-2 w-full rounded-md px-2.5 py-1.5 text-xs font-medium"
          onClick={() => void pickProject()}
        >
          选择目录创建项目
        </button>
        <div className="max-h-52 space-y-1 overflow-y-auto">
          {projects.map((project) => (
            <div
              key={project.id}
              className={`group flex items-stretch rounded-md border text-xs ${
                project.id === activeProjectId ? "item-project-active" : "item-surface"
              }`}
            >
              <button
                className="min-w-0 flex-1 px-2.5 py-1.5 text-left"
                onClick={() => void onSelectProject(project.id)}
              >
                <div className="font-medium">{project.name}</div>
                <div className="truncate text-[11px] text-fg-secondary">{project.root_path}</div>
              </button>
              <button
                type="button"
                className="shrink-0 border-l border-transparent px-2 text-fg-muted opacity-0 transition hover:text-rose-400 group-hover:border-border-subtle group-hover:opacity-100"
                title="从列表移除"
                onClick={() => void hideProject(project.id)}
              >
                ×
              </button>
            </div>
          ))}
        </div>
      </div>

      <div className="flex min-h-0 flex-1 flex-col">
        <div className="mb-1 flex shrink-0 items-center justify-between">
          <div className="text-[11px] uppercase tracking-[0.16em] text-fg-secondary">会话</div>
          <button
            className="rounded border border-border-subtle px-1.5 py-0.5 text-[11px] hover:border-border-hover"
            onClick={() => void onCreateSession()}
            disabled={!activeProjectId}
          >
            新建
          </button>
        </div>
        <SessionList
          sessions={sessions}
          activeSessionId={activeSessionId}
          onSelectSession={onSelectSession}
          onDeleteSession={(sessionId) => void onDeleteSession(sessionId)}
          onReorderSessions={onReorderSessions}
        />
      </div>

      {activeProjectId && (
        <div className="shrink-0 space-y-1 border-t border-border pt-2.5">
          <div className="text-[11px] uppercase tracking-[0.16em] text-fg-secondary">模型</div>
          <button
            type="button"
            className="config-surface w-full rounded-md px-2.5 py-2 text-left text-xs text-fg hover:border-border-hover"
            onClick={onOpenModelSettings}
          >
            <div>{modelSummary}</div>
            <div className="mt-0.5 text-[10px] text-fg-muted">
              {modelLocked ? "已锁定 · 点击查看密钥" : "点击配置模型与密钥"}
            </div>
          </button>
        </div>
      )}

      <div className="mt-auto shrink-0 space-y-1.5">
        <WebSearchSection enabled={tavilyEnabled} onStatusChange={onTavilyStatusChange} />
      </div>
    </aside>
  );
}
