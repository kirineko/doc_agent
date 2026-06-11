import { LiveToolCall, ToolChainPanel } from "./ToolChainPanel";
import { ProjectFileExplorer } from "./ProjectFileExplorer";

interface RightPanelProps {
  liveTools: LiveToolCall[];
  projectId?: string;
}

export function RightPanel({ liveTools, projectId }: RightPanelProps) {
  return (
    <aside className="panel flex h-full w-64 shrink-0 flex-col p-2.5">
      <ToolChainPanel items={liveTools} />
      <ProjectFileExplorer projectId={projectId} />
    </aside>
  );
}
