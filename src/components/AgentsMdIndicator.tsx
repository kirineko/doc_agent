import type { AgentsMdStatus } from "../lib/agentsMdStatus";
import { agentsMdStatusHint, agentsMdStatusLabel } from "../lib/agentsMdStatus";

interface AgentsMdIndicatorProps {
  status: AgentsMdStatus;
  /** compact: dot only with tooltip; labeled: dot + short text */
  variant?: "compact" | "labeled";
  className?: string;
}

const DOT_CLASS: Record<AgentsMdStatus, string> = {
  idle: "bg-fg-muted/30",
  loading: "bg-amber-400 animate-pulse",
  missing: "bg-fg-muted/50 ring-1 ring-fg-muted/40",
  loaded: "bg-emerald-500 shadow-[0_0_6px_rgba(16,185,129,0.55)]",
};

export function AgentsMdIndicator({
  status,
  variant = "compact",
  className = "",
}: AgentsMdIndicatorProps) {
  if (status === "idle") return null;

  const label = agentsMdStatusLabel(status);
  const hint = agentsMdStatusHint(status);

  return (
    <span
      className={`inline-flex items-center gap-1.5 text-[10px] text-fg-muted ${className}`}
      title={hint}
      aria-label={label}
    >
      <span
        className={`inline-block h-2 w-2 shrink-0 rounded-full ${DOT_CLASS[status]}`}
        aria-hidden
      />
      {variant === "labeled" && <span className="truncate">{label}</span>}
    </span>
  );
}
