import { useState } from "react";
import { ChevronDownIcon, ChevronRightIcon, panelIconButtonClassName } from "./PanelIcons";
import { ToolChainPanel, LiveToolCall } from "./ToolChainPanel";
import { BuildArtifactsPanel } from "./BuildArtifactsPanel";
import { TurnArtifact } from "../lib/agentEvents";

type TabKey = "toolchain" | "artifacts";

interface ToolchainTabAreaProps {
  liveTools: LiveToolCall[];
  turnArtifacts: TurnArtifact[];
  projectId?: string;
  collapsed: boolean;
  onToggleCollapse: () => void;
}

const TAB_BTN_BASE =
  "shrink-0 rounded px-1.5 py-0.5 text-[11px] font-medium transition-colors";
const tabBtnClass = (active: boolean) =>
  active
    ? `${TAB_BTN_BASE} bg-hover text-fg-heading`
    : `${TAB_BTN_BASE} text-fg-secondary hover:text-fg-heading`;

export function ToolchainTabArea({
  liveTools,
  turnArtifacts,
  projectId,
  collapsed,
  onToggleCollapse,
}: ToolchainTabAreaProps) {
  const [activeTab, setActiveTab] = useState<TabKey>("toolchain");
  const artifactCount = turnArtifacts.length;

  return (
    <section className="flex h-full min-h-0 flex-col">
      <div className="mb-1.5 flex shrink-0 items-center justify-between gap-2">
        {collapsed ? (
          <div className="min-w-0 truncate text-xs font-medium text-fg-heading">工具调用链</div>
        ) : (
          <div className="flex min-w-0 items-center gap-1">
            <button
              type="button"
              className={tabBtnClass(activeTab === "toolchain")}
              onClick={() => setActiveTab("toolchain")}
            >
              工具调用链
            </button>
            <button
              type="button"
              className={tabBtnClass(activeTab === "artifacts")}
              onClick={() => setActiveTab("artifacts")}
            >
              构建产物
              {artifactCount > 0 && (
                <span className="ml-1 rounded-full bg-sky-500/15 px-1.5 text-[10px] text-sky-600 dark:text-sky-400">
                  {artifactCount}
                </span>
              )}
            </button>
          </div>
        )}
        <button
          type="button"
          className={panelIconButtonClassName()}
          aria-label={collapsed ? "展开工具调用链" : "折叠工具调用链"}
          title={collapsed ? "展开工具调用链" : "折叠工具调用链"}
          onClick={onToggleCollapse}
        >
          {collapsed ? <ChevronRightIcon /> : <ChevronDownIcon />}
        </button>
      </div>
      {!collapsed &&
        (activeTab === "toolchain" ? (
          <ToolChainPanel items={liveTools} hideHeader />
        ) : (
          <BuildArtifactsPanel artifacts={turnArtifacts} projectId={projectId} />
        ))}
    </section>
  );
}
