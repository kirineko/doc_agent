import type { SessionRunStatus } from "../lib/sessionRunState";
import type { Project, Session } from "../types";
import { ProjectSessionTree } from "./ProjectSessionTree";
import { WebSearchStatus } from "./WebSearchStatus";

interface SidebarProps {
  projects: Project[];
  sessions: Session[];
  activeProjectId?: string;
  activeSessionId?: string;
  sessionRunStatuses?: Record<string, SessionRunStatus>;
  highlightProject?: boolean;
  webSearchActive: boolean;
  onProjectsChange: (projects: Project[]) => void;
  onSelectProject: (projectId: string | undefined) => void | Promise<void>;
  onSelectSession: (sessionId: string | undefined) => void;
  onCreateSession: (projectId?: string) => void | Promise<void>;
  onDeleteSession: (sessionId: string) => void | Promise<void>;
  onReorderSessions: (activeId: string, overId: string) => void;
  onEnableWebSearch: () => void;
  onDisableWebSearch: () => void;
  onOpenCommandPalette: () => void;
  onPromptAddProject: () => void;
  onAddProject: () => void | Promise<void>;
}

export function Sidebar({
  projects,
  sessions,
  activeProjectId,
  activeSessionId,
  sessionRunStatuses,
  highlightProject,
  webSearchActive,
  onProjectsChange,
  onSelectProject,
  onSelectSession,
  onCreateSession,
  onDeleteSession,
  onReorderSessions,
  onEnableWebSearch,
  onDisableWebSearch,
  onOpenCommandPalette,
  onPromptAddProject,
  onAddProject,
}: SidebarProps) {
  return (
    <aside className="workspace-pane flex h-full min-h-0 flex-col gap-2 border-r border-border-subtle px-3 py-2.5">
      <ProjectSessionTree
        projects={projects}
        sessions={sessions}
        activeProjectId={activeProjectId}
        activeSessionId={activeSessionId}
        sessionRunStatuses={sessionRunStatuses}
        highlightProject={highlightProject}
        onProjectsChange={onProjectsChange}
        onSelectProject={onSelectProject}
        onSelectSession={onSelectSession}
        onCreateSession={onCreateSession}
        onDeleteSession={onDeleteSession}
        onReorderSessions={onReorderSessions}
        onOpenSearch={onOpenCommandPalette}
        onPromptAddProject={onPromptAddProject}
        onAddProject={onAddProject}
      />

      <div className="mt-auto shrink-0 space-y-1.5 border-t border-border-subtle pt-2.5">
        <WebSearchStatus
          enabled={webSearchActive}
          onEnable={() => void onEnableWebSearch()}
          onDisable={() => void onDisableWebSearch()}
        />
      </div>
    </aside>
  );
}
