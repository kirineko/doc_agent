import type { ReactNode } from "react";
import { ChevronDownIcon, ChevronRightIcon, panelIconButtonClassName } from "./PanelIcons";

interface PanelSectionHeaderProps {
  title: string;
  collapsed: boolean;
  onToggleCollapse: () => void;
  actions?: ReactNode;
}

export function PanelSectionHeader({
  title,
  collapsed,
  onToggleCollapse,
  actions,
}: PanelSectionHeaderProps) {
  return (
    <div className="mb-1.5 flex shrink-0 items-center justify-between gap-2">
      <div className="min-w-0 truncate text-xs font-medium text-fg-heading">{title}</div>
      <div className="flex shrink-0 items-center gap-1">
        {actions}
        <button
          type="button"
          className={panelIconButtonClassName()}
          aria-label={collapsed ? `展开${title}` : `折叠${title}`}
          title={collapsed ? `展开${title}` : `折叠${title}`}
          onClick={onToggleCollapse}
        >
          {collapsed ? <ChevronRightIcon /> : <ChevronDownIcon />}
        </button>
      </div>
    </div>
  );
}
