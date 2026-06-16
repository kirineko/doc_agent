import { useCallback, useEffect, useRef, useState } from "react";
import { Group, Panel, useDefaultLayout, useGroupRef, usePanelRef } from "react-resizable-panels";
import {
  applyRightPanelCollapseMode,
  normalizeRightPanelCollapseMode,
  rightPanelCollapseFlags,
  shouldExpandBothOnLayoutChange,
  toggleRightPanelCollapseMode,
  type RightPanelCollapseMode,
} from "../lib/rightPanelCollapse";
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
  const collapseModeRef = useRef<RightPanelCollapseMode>("both");
  const isApplyingCollapseRef = useRef(false);
  const [collapseMode, setCollapseMode] = useState<RightPanelCollapseMode>("both");

  const { defaultLayout, onLayoutChanged } = useDefaultLayout({
    id: RIGHT_LAYOUT_GROUP_ID,
    storage: rightLayoutStorage,
    panelIds: [RIGHT_PANEL_IDS.toolchain, RIGHT_PANEL_IDS.files],
  });

  const applyCollapseMode = useCallback(
    (mode: RightPanelCollapseMode) => {
      collapseModeRef.current = mode;
      setCollapseMode(mode);

      const toolchainPanel = toolchainRef.current;
      const filesPanel = filesRef.current;
      if (!toolchainPanel || !filesPanel) return;

      isApplyingCollapseRef.current = true;
      applyRightPanelCollapseMode(toolchainPanel, filesPanel, mode);
      requestAnimationFrame(() => {
        isApplyingCollapseRef.current = false;
      });
    },
    [filesRef, toolchainRef],
  );

  const expandBothFromLayoutInteraction = useCallback(() => {
    if (!shouldExpandBothOnLayoutChange(collapseModeRef.current, isApplyingCollapseRef.current)) {
      return;
    }

    collapseModeRef.current = "both";
    setCollapseMode("both");

    const toolchainPanel = toolchainRef.current;
    const filesPanel = filesRef.current;
    if (!toolchainPanel || !filesPanel) return;

    toolchainPanel.expand();
    filesPanel.expand();
  }, [filesRef, toolchainRef]);

  const toggleToolchain = useCallback(() => {
    applyCollapseMode(toggleRightPanelCollapseMode(collapseModeRef.current, "toolchain"));
  }, [applyCollapseMode]);

  const toggleFiles = useCallback(() => {
    applyCollapseMode(toggleRightPanelCollapseMode(collapseModeRef.current, "files"));
  }, [applyCollapseMode]);

  const handleLayoutChange = useCallback(() => {
    expandBothFromLayoutInteraction();
  }, [expandBothFromLayoutInteraction]);

  const handleLayoutChanged = useCallback(
    (layout: Record<string, number>) => {
      expandBothFromLayoutInteraction();
      onLayoutChanged(layout);
    },
    [expandBothFromLayoutInteraction, onLayoutChanged],
  );

  useEffect(() => {
    const toolchainPanel = toolchainRef.current;
    const filesPanel = filesRef.current;
    if (!toolchainPanel || !filesPanel) return;

    const mode = normalizeRightPanelCollapseMode({
      toolchainCollapsed: toolchainPanel.isCollapsed(),
      filesCollapsed: filesPanel.isCollapsed(),
    });
    collapseModeRef.current = mode;
    setCollapseMode(mode);

    if (mode !== "both") {
      isApplyingCollapseRef.current = true;
      applyRightPanelCollapseMode(toolchainPanel, filesPanel, mode);
      requestAnimationFrame(() => {
        isApplyingCollapseRef.current = false;
      });
    }
  }, [filesRef, toolchainRef]);

  useEffect(() => {
    return onWorkspaceLayoutReset(() => {
      toolchainRef.current?.expand();
      filesRef.current?.expand();
      groupRef.current?.setLayout(DEFAULT_RIGHT_LAYOUT);
      collapseModeRef.current = "both";
      setCollapseMode("both");
    });
  }, [groupRef, toolchainRef, filesRef]);

  const { toolchainCollapsed, filesCollapsed } = rightPanelCollapseFlags(collapseMode);

  return (
    <aside className="panel flex h-full min-h-0 flex-col p-2.5">
      <Group
        id={RIGHT_LAYOUT_GROUP_ID}
        groupRef={groupRef}
        orientation="vertical"
        className="h-full min-h-0"
        defaultLayout={defaultLayout ?? DEFAULT_RIGHT_LAYOUT}
        onLayoutChange={handleLayoutChange}
        onLayoutChanged={handleLayoutChanged}
      >
        <Panel
          id={RIGHT_PANEL_IDS.toolchain}
          panelRef={toolchainRef}
          defaultSize={PANEL_DEFAULT_SIZE.toolchain}
          minSize={PANEL_MIN_SIZE.toolchain}
          collapsedSize={PANEL_COLLAPSED_SIZE}
          collapsible
          className="min-h-0"
        >
          <ToolChainPanel
            items={liveTools}
            collapsed={toolchainCollapsed}
            onToggleCollapse={toggleToolchain}
          />
        </Panel>
        <PanelSeparator orientation="vertical" />
        <Panel
          id={RIGHT_PANEL_IDS.files}
          panelRef={filesRef}
          defaultSize={PANEL_DEFAULT_SIZE.files}
          minSize={PANEL_MIN_SIZE.files}
          collapsedSize={PANEL_COLLAPSED_SIZE}
          collapsible
          className="min-h-0"
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
