import { useEffect, useRef, useState } from "react";
import type { SessionConfig } from "../lib/sessionConfig";
import type { AgentsMdStatus } from "../lib/agentsMdStatus";
import type { ModelInfo, Project } from "../types";
import { AgentsMdIndicator } from "./AgentsMdIndicator";
import { ContextUsageIndicator } from "./ContextUsageIndicator";
import { ModelFlyout } from "./ModelFlyout";

interface ComposerContextBarProps {
  projectId?: string;
  projects: Project[];
  models: ModelInfo[];
  sessionConfig: SessionConfig;
  modelLocked: boolean;
  modelSummary: string;
  apiKeyStatus: Record<string, boolean>;
  agentsMdStatus: AgentsMdStatus;
  contextRatio: number;
  onSelectProject: (projectId: string) => void | Promise<void>;
  onSessionConfigChange: (patch: Partial<SessionConfig>) => void;
  onModelFlyoutOpenChange?: (open: boolean) => void;
}

export function ComposerContextBar({
  projectId,
  projects,
  models,
  sessionConfig,
  modelLocked,
  modelSummary,
  apiKeyStatus,
  agentsMdStatus,
  contextRatio,
  onSelectProject,
  onSessionConfigChange,
  onModelFlyoutOpenChange,
}: ComposerContextBarProps) {
  const modelTriggerRef = useRef<HTMLButtonElement>(null);
  const projectMenuRef = useRef<HTMLDivElement>(null);
  const [modelFlyoutOpen, setModelFlyoutOpen] = useState(false);
  const [projectMenuOpen, setProjectMenuOpen] = useState(false);

  const activeProject = projects.find((item) => item.id === projectId);

  useEffect(() => {
    onModelFlyoutOpenChange?.(modelFlyoutOpen);
  }, [modelFlyoutOpen, onModelFlyoutOpenChange]);

  useEffect(() => {
    if (!projectMenuOpen) return;
    function handlePointerDown(event: MouseEvent) {
      if (projectMenuRef.current?.contains(event.target as Node)) return;
      setProjectMenuOpen(false);
    }
    window.addEventListener("mousedown", handlePointerDown);
    return () => window.removeEventListener("mousedown", handlePointerDown);
  }, [projectMenuOpen]);

  if (!projectId) return null;

  return (
    <div className="composer-context-bar mb-2 flex flex-wrap items-center gap-2 text-xs">
      <div className="relative" ref={projectMenuRef}>
        <button
          type="button"
          className="inline-flex items-center gap-1 rounded-md px-2 py-1 font-medium text-fg hover:bg-hover"
          aria-expanded={projectMenuOpen}
          onClick={() => setProjectMenuOpen((open) => !open)}
        >
          <span className="max-w-[10rem] truncate">{activeProject?.name ?? "项目"}</span>
          <span className="text-fg-muted" aria-hidden>
            ▾
          </span>
        </button>
        {projectMenuOpen && (
          <div className="absolute bottom-full left-0 z-30 mb-1 max-h-48 min-w-[12rem] overflow-y-auto rounded-md border border-border-subtle bg-elevated py-1 shadow-lg">
            {projects.map((project) => (
              <button
                key={project.id}
                type="button"
                className={`block w-full px-3 py-1.5 text-left text-xs hover:bg-hover ${
                  project.id === projectId ? "font-medium text-fg-heading" : "text-fg"
                }`}
                onClick={() => {
                  setProjectMenuOpen(false);
                  void onSelectProject(project.id);
                }}
              >
                {project.name}
              </button>
            ))}
          </div>
        )}
      </div>

      <span className="text-fg-muted" aria-hidden>
        |
      </span>

      <div className="relative">
        <button
          id="composer-model-trigger"
          ref={modelTriggerRef}
          type="button"
          className="inline-flex max-w-[14rem] items-center gap-1 truncate rounded-md px-2 py-1 text-fg hover:bg-hover"
          aria-expanded={modelFlyoutOpen}
          onClick={() => setModelFlyoutOpen((open) => !open)}
        >
          <span className="truncate">{modelSummary}</span>
          <span className="shrink-0 text-fg-muted" aria-hidden>
            ▾
          </span>
        </button>
        <ModelFlyout
          open={modelFlyoutOpen}
          triggerRef={modelTriggerRef}
          models={models}
          config={sessionConfig}
          locked={modelLocked}
          apiKeyStatus={apiKeyStatus}
          onClose={() => setModelFlyoutOpen(false)}
          onChange={onSessionConfigChange}
        />
      </div>

      <AgentsMdIndicator status={agentsMdStatus} variant="labeled" />

      <div className="ml-auto">
        <ContextUsageIndicator ratio={contextRatio} />
      </div>
    </div>
  );
}
