export type RightPanelCollapseMode = "both" | "toolchain" | "files";

export interface PanelCollapseController {
  collapse: () => void;
  expand: () => void;
}

export interface RightPanelCollapseFlags {
  toolchainCollapsed: boolean;
  filesCollapsed: boolean;
}

export function rightPanelCollapseFlags(mode: RightPanelCollapseMode): RightPanelCollapseFlags {
  return {
    toolchainCollapsed: mode === "toolchain",
    filesCollapsed: mode === "files",
  };
}

/** Never allow both sections collapsed; prefer keeping toolchain expanded. */
export function normalizeRightPanelCollapseMode(flags: RightPanelCollapseFlags): RightPanelCollapseMode {
  if (flags.toolchainCollapsed && flags.filesCollapsed) return "files";
  if (flags.toolchainCollapsed) return "toolchain";
  if (flags.filesCollapsed) return "files";
  return "both";
}

export function toggleRightPanelCollapseMode(
  mode: RightPanelCollapseMode,
  target: "toolchain" | "files",
): RightPanelCollapseMode {
  if (target === "toolchain") {
    return mode === "toolchain" ? "both" : "toolchain";
  }
  return mode === "files" ? "both" : "files";
}

export function shouldExpandBothOnLayoutChange(
  mode: RightPanelCollapseMode,
  isApplyingCollapse: boolean,
): boolean {
  return !isApplyingCollapse && mode !== "both";
}

/** Expand the visible section first, then collapse the other to avoid a dual-collapsed frame. */
export function applyRightPanelCollapseMode(
  toolchainPanel: PanelCollapseController,
  filesPanel: PanelCollapseController,
  mode: RightPanelCollapseMode,
): void {
  switch (mode) {
    case "both":
      toolchainPanel.expand();
      filesPanel.expand();
      break;
    case "toolchain":
      filesPanel.expand();
      toolchainPanel.collapse();
      break;
    case "files":
      toolchainPanel.expand();
      filesPanel.collapse();
      break;
  }
}
