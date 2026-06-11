import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { plainSessionTitle } from "../lib/formatTitle";
import { formatSessionTime } from "../lib/formatTime";
import { pathBasename } from "../lib/pathUtils";
import { buildCreateSessionRequest, type SessionConfig } from "../lib/sessionConfig";
import { Project, Session } from "../types";
import { ApiKeySection } from "./ApiKeySection";
import { ModelConfigSection } from "./ModelConfigSection";
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
  onSessionsChange: (sessions: Session[]) => void;
  onSelectProject: (projectId: string | undefined) => void | Promise<void>;
  onSelectSession: (sessionId: string | undefined) => void;
  onPendingSessionConfigChange: (patch: Partial<SessionConfig>) => void;
  onSessionUpdated: (session: Session) => void;
  onApiKeyStatusChange: (provider: string, has: boolean) => void;
  onTavilyStatusChange: (has: boolean) => void;
}

export function Sidebar({
  projects,
  sessions,
  activeProjectId,
  activeSessionId,
  apiKeyStatus,
  pendingSessionConfig,
  modelLocked,
  highlightProject,
  highlightApiKeyProvider,
  tavilyEnabled,
  onProjectsChange,
  onSessionsChange,
  onSelectProject,
  onSelectSession,
  onPendingSessionConfigChange,
  onSessionUpdated,
  onApiKeyStatusChange,
  onTavilyStatusChange,
}: SidebarProps) {
  const activeSession = sessions.find((s) => s.id === activeSessionId);
  const effectiveConfig: SessionConfig = activeSession
    ? {
        model: activeSession.model,
        thinking_enabled: activeSession.thinking_enabled,
        thinking_effort: activeSession.thinking_effort,
      }
    : pendingSessionConfig;

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

  async function createSession() {
    if (!activeProjectId) return;
    const session = await invoke<Session>("create_session", {
      req: buildCreateSessionRequest(activeProjectId, pendingSessionConfig),
    });
    onSessionsChange([session, ...sessions]);
    onSelectSession(session.id);
  }

  async function updateSessionConfig(patch: Partial<SessionConfig>) {
    if (modelLocked) return;
    if (activeSession) {
      try {
        const updated = await invoke<Session>("update_session", {
          req: {
            session_id: activeSession.id,
            model: patch.model,
            thinking_enabled: patch.thinking_enabled,
            thinking_effort: patch.thinking_effort,
          },
        });
        onSessionUpdated(updated);
      } catch (error) {
        console.error(error);
      }
      return;
    }
    onPendingSessionConfigChange(patch);
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

  async function deleteSession(sessionId: string) {
    try {
      await invoke("delete_session", { sessionId });
      const next = sessions.filter((item) => item.id !== sessionId);
      onSessionsChange(next);
      if (activeSessionId === sessionId) {
        onSelectSession(next[0]?.id);
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
          className="mb-2 w-full rounded-md bg-indigo-600 px-2.5 py-1.5 text-xs font-medium text-white hover:bg-indigo-500"
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
            onClick={() => void createSession()}
            disabled={!activeProjectId}
          >
            新建
          </button>
        </div>
        <div className="min-h-0 flex-1 space-y-1 overflow-y-auto">
          {sessions.map((session) => (
            <div
              key={session.id}
              className={`group relative rounded-md border text-xs ${
                session.id === activeSessionId ? "item-session-active" : "item-surface"
              }`}
            >
              <button
                className="w-full px-2.5 py-1.5 pr-7 text-left"
                onClick={() => onSelectSession(session.id)}
              >
                <div className="truncate font-medium">{plainSessionTitle(session.title)}</div>
                <div className="text-[11px] text-fg-secondary">{formatSessionTime(session.updated_at)}</div>
              </button>
              <button
                type="button"
                className="absolute right-1 top-1/2 -translate-y-1/2 rounded px-1.5 text-fg-muted opacity-0 transition hover:text-rose-400 group-hover:opacity-100"
                title="删除会话"
                onClick={() => void deleteSession(session.id)}
              >
                ×
              </button>
            </div>
          ))}
        </div>
      </div>

      {activeProjectId && (
        <ModelConfigSection
          config={effectiveConfig}
          locked={modelLocked}
          onChange={(patch) => void updateSessionConfig(patch)}
        />
      )}

      <div className="mt-auto shrink-0 space-y-1.5">
        <WebSearchSection enabled={tavilyEnabled} onStatusChange={onTavilyStatusChange} />
        <ApiKeySection
          apiKeyStatus={apiKeyStatus}
          highlightProvider={highlightApiKeyProvider}
          onApiKeyStatusChange={onApiKeyStatusChange}
        />
      </div>
    </aside>
  );
}
