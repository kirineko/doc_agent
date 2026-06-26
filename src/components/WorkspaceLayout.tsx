import type { ReactNode } from "react";
import { useEffect } from "react";
import { Group, Panel, useDefaultLayout, useGroupRef } from "react-resizable-panels";
import {
  DEFAULT_MAIN_LAYOUT,
  MAIN_LAYOUT_GROUP_ID,
  MAIN_PANEL_IDS,
  mainLayoutStorage,
  onWorkspaceLayoutReset,
  PANEL_DEFAULT_SIZE,
  PANEL_MIN_SIZE,
} from "../lib/workspaceLayout";
import { PanelSeparator } from "./PanelSeparator";

interface WorkspaceLayoutProps {
  sidebar: ReactNode;
  chat: ReactNode;
  right: ReactNode;
}

export function WorkspaceLayout({ sidebar, chat, right }: WorkspaceLayoutProps) {
  const groupRef = useGroupRef();
  const { defaultLayout, onLayoutChanged } = useDefaultLayout({
    id: MAIN_LAYOUT_GROUP_ID,
    storage: mainLayoutStorage,
    panelIds: [MAIN_PANEL_IDS.sidebar, MAIN_PANEL_IDS.chat, MAIN_PANEL_IDS.right],
  });

  useEffect(() => {
    return onWorkspaceLayoutReset(() => {
      groupRef.current?.setLayout(DEFAULT_MAIN_LAYOUT);
    });
  }, [groupRef]);

  return (
    <main className="min-h-0 flex-1">
      <Group
        id={MAIN_LAYOUT_GROUP_ID}
        groupRef={groupRef}
        orientation="horizontal"
        className="h-full min-h-0"
        defaultLayout={defaultLayout ?? DEFAULT_MAIN_LAYOUT}
        onLayoutChanged={onLayoutChanged}
      >
        <Panel
          id={MAIN_PANEL_IDS.sidebar}
          defaultSize={PANEL_DEFAULT_SIZE.sidebar}
          minSize={PANEL_MIN_SIZE.sidebar}
          className="min-h-0"
        >
          <div className="h-full min-h-0">{sidebar}</div>
        </Panel>
        <PanelSeparator orientation="horizontal" />
        <Panel id={MAIN_PANEL_IDS.chat} minSize={PANEL_MIN_SIZE.chat} className="min-h-0">
          <div className="flex h-full min-h-0 flex-col">{chat}</div>
        </Panel>
        <PanelSeparator orientation="horizontal" />
        <Panel
          id={MAIN_PANEL_IDS.right}
          defaultSize={PANEL_DEFAULT_SIZE.right}
          minSize={PANEL_MIN_SIZE.right}
          className="min-h-0"
        >
          <div className="h-full min-h-0">{right}</div>
        </Panel>
      </Group>
    </main>
  );
}
