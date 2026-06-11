import { LiveToolCall, ToolChainPanel } from "./ToolChainPanel";
import { ProjectFileExplorer } from "./ProjectFileExplorer";

interface RightPanelProps {
  liveTools: LiveToolCall[];
  projectId?: string;
  fileRevision?: number;
}

export function RightPanel({ liveTools, projectId, fileRevision }: RightPanelProps) {
  return (
    <aside className="panel flex h-full w-64 shrink-0 flex-col p-2.5">
      <ToolChainPanel items={liveTools} />
      <ProjectFileExplorer projectId={projectId} fileRevision={fileRevision} />
    </aside>
  );
}
