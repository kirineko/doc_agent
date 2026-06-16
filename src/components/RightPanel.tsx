import { useCallback, useEffect, useState } from "react";
import { Group, Panel, useDefaultLayout, useGroupRef, usePanelRef } from "react-resizable-panels";
import {
  DEFAULT_RIGHT_LAYOUT,
  onWorkspaceLayoutReset,
  PANEL_COLLAPSED_SIZE,
  PANEL_DEFAULT_SIZE,
  PANEL_MIN_SIZE,
  rightLayoutStorage,
  RIGHT_LAYOUT_GROUP_ID,
  RIGHT_PANEL_IDS,
} from "../lib/workspaceLayout";
import { ProjectFileExplorer } from "./ProjectFileExplorer";
import { PanelSeparator } from "./PanelSeparator";
import { LiveToolCall, ToolChainPanel } from "./ToolChainPanel";

interface RightPanelProps {
  liveTools: LiveToolCall[];
  projectId?: string;
  fileRevision?: number;
}

export function RightPanel({ liveTools, projectId, fileRevision }: RightPanelProps) {
  const groupRef = useGroupRef();
  const toolchainRef = usePanelRef();
  const filesRef = usePanelRef();
  const [toolchainCollapsed, setToolchainCollapsed] = useState(false);
  const [filesCollapsed, setFilesCollapsed] = useState(false);

  const { defaultLayout, onLayoutChanged } = useDefaultLayout({
    id: RIGHT_LAYOUT_GROUP_ID,
    storage: rightLayoutStorage,
    panelIds: [RIGHT_PANEL_IDS.toolchain, RIGHT_PANEL_IDS.files],
  });

  const syncCollapsedState = useCallback(() => {
    setToolchainCollapsed(toolchainRef.current?.isCollapsed() ?? false);
    setFilesCollapsed(filesRef.current?.isCollapsed() ?? false);
  }, [toolchainRef, filesRef]);

  const toggleToolchain = useCallback(() => {
    const panel = toolchainRef.current;
    if (!panel) return;
    if (panel.isCollapsed()) panel.expand();
    else panel.collapse();
    requestAnimationFrame(syncCollapsedState);
  }, [toolchainRef, syncCollapsedState]);

  const toggleFiles = useCallback(() => {
    const panel = filesRef.current;
    if (!panel) return;
    if (panel.isCollapsed()) panel.expand();
    else panel.collapse();
    requestAnimationFrame(syncCollapsedState);
  }, [filesRef, syncCollapsedState]);

  useEffect(() => {
    syncCollapsedState();
  }, [syncCollapsedState]);

  useEffect(() => {
    return onWorkspaceLayoutReset(() => {
      toolchainRef.current?.expand();
      filesRef.current?.expand();
      groupRef.current?.setLayout(DEFAULT_RIGHT_LAYOUT);
      setToolchainCollapsed(false);
      setFilesCollapsed(false);
    });
  }, [groupRef, toolchainRef, filesRef]);

  const hideVerticalSeparator = toolchainCollapsed || filesCollapsed;

  return (
    <aside className="panel flex h-full min-h-0 flex-col p-2.5">
      <Group
        id={RIGHT_LAYOUT_GROUP_ID}
        groupRef={groupRef}
        orientation="vertical"
        className="h-full min-h-0"
        defaultLayout={defaultLayout ?? DEFAULT_RIGHT_LAYOUT}
        onLayoutChanged={onLayoutChanged}
      >
        <Panel
          id={RIGHT_PANEL_IDS.toolchain}
          panelRef={toolchainRef}
          defaultSize={PANEL_DEFAULT_SIZE.toolchain}
          minSize={PANEL_MIN_SIZE.toolchain}
          collapsedSize={PANEL_COLLAPSED_SIZE}
          collapsible
          className="min-h-0"
          onResize={() => syncCollapsedState()}
        >
          <ToolChainPanel
            items={liveTools}
            collapsed={toolchainCollapsed}
            onToggleCollapse={toggleToolchain}
          />
        </Panel>
        <PanelSeparator orientation="vertical" disabled={hideVerticalSeparator} />
        <Panel
          id={RIGHT_PANEL_IDS.files}
          panelRef={filesRef}
          defaultSize={PANEL_DEFAULT_SIZE.files}
          minSize={PANEL_MIN_SIZE.files}
          collapsedSize={PANEL_COLLAPSED_SIZE}
          collapsible
          className="min-h-0"
          onResize={() => syncCollapsedState()}
        >
          <ProjectFileExplorer
            projectId={projectId}
            fileRevision={fileRevision}
            collapsed={filesCollapsed}
            onToggleCollapse={toggleFiles}
          />
        </Panel>
      </Group>
    </aside>
  );
}
