import { useInspectorTab } from "../hooks/useInspectorTab";
import { BuildArtifactsPanel } from "./BuildArtifactsPanel";
import { ProjectFileExplorer } from "./ProjectFileExplorer";
import { LiveToolCall, ToolChainPanel } from "./ToolChainPanel";
import { TurnArtifact } from "../lib/agentEvents";

interface InspectorTabsProps {
  liveTools: LiveToolCall[];
  turnArtifacts: TurnArtifact[];
  projectId?: string;
  fileRevision?: number;
  inspectorTurnNonce?: number;
  activeSessionId?: string;
}

const TAB_BTN_BASE =
  "shrink-0 rounded-md px-2 py-1 text-[11px] font-medium transition-colors";
const tabBtnClass = (active: boolean) =>
  active
    ? `${TAB_BTN_BASE} bg-hover text-fg-heading`
    : `${TAB_BTN_BASE} text-fg-secondary hover:bg-hover hover:text-fg-heading`;

export function InspectorTabs({
  liveTools,
  turnArtifacts,
  projectId,
  fileRevision,
  inspectorTurnNonce = 0,
  activeSessionId,
}: InspectorTabsProps) {
  const { tab, setTab } = useInspectorTab({
    liveTools,
    turnNonce: inspectorTurnNonce,
    activeSessionId,
  });
  const artifactCount = turnArtifacts.length;

  return (
    <section className="workspace-pane flex h-full min-h-0 flex-col px-3 py-2.5">
      <div className="mb-2 flex shrink-0 items-center gap-1 border-b border-border-subtle pb-2">
        <button
          type="button"
          className={tabBtnClass(tab === "files")}
          onClick={() => setTab("files", true)}
        >
          项目文件
        </button>
        <button
          type="button"
          className={tabBtnClass(tab === "toolchain")}
          onClick={() => setTab("toolchain", true)}
        >
          工具调用链
        </button>
        <button
          type="button"
          className={tabBtnClass(tab === "artifacts")}
          onClick={() => setTab("artifacts", true)}
        >
          构建产物
          {artifactCount > 0 && (
            <span className="ml-1 rounded-full bg-sky-500/15 px-1.5 text-[10px] text-sky-600 dark:text-sky-400">
              {artifactCount}
            </span>
          )}
        </button>
      </div>
      <div className="relative min-h-0 flex-1">
        <div className={tab === "files" ? "h-full" : "hidden"} aria-hidden={tab !== "files"}>
          <ProjectFileExplorer projectId={projectId} fileRevision={fileRevision} />
        </div>
        <div className={tab === "toolchain" ? "h-full" : "hidden"} aria-hidden={tab !== "toolchain"}>
          <ToolChainPanel items={liveTools} />
        </div>
        <div className={tab === "artifacts" ? "h-full" : "hidden"} aria-hidden={tab !== "artifacts"}>
          <BuildArtifactsPanel artifacts={turnArtifacts} projectId={projectId} />
        </div>
      </div>
    </section>
  );
}
