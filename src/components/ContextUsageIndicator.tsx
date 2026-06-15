interface ContextUsageIndicatorProps {
  ratio?: number;
  hidden?: boolean;
}

function toneClass(ratio: number): string {
  if (ratio >= 0.85) return "text-red-500";
  if (ratio >= 0.7) return "text-amber-600";
  return "text-fg-secondary";
}

export function ContextUsageIndicator({ ratio = 0, hidden }: ContextUsageIndicatorProps) {
  if (hidden) return null;

  const percent = Math.min(100, Math.max(0, Math.round(ratio * 100)));

  return (
    <span
      className={`inline-flex items-center gap-1 text-xs tabular-nums ${toneClass(ratio)}`}
      title="上下文占用比例"
      aria-label={`上下文占用 ${percent}%`}
    >
      <svg
        viewBox="0 0 16 16"
        className="h-3.5 w-3.5"
        aria-hidden="true"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
      >
        <circle cx="8" cy="8" r="6" opacity="0.35" />
        <path d="M8 2 A6 6 0 0 1 8 14" strokeLinecap="round" />
      </svg>
      {percent}%
    </span>
  );
}
