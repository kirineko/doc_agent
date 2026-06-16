import { computeUpdatePercent } from "../lib/updateProgress";
import { useUpdateProgress } from "../hooks/useUpdateProgress";

const RING_RADIUS = 20;
const RING_CIRCUMFERENCE = 2 * Math.PI * RING_RADIUS;

export interface UpdateOverlayCopy {
  primary: string;
  secondary?: string;
  percentLine?: string;
}

export function formatUpdateOverlayCopy(
  phase: "downloading" | "installing",
  version?: string,
  percent?: number,
): UpdateOverlayCopy {
  if (phase === "installing") {
    return { primary: "正在安装", secondary: "即将重启" };
  }

  if (percent !== undefined) {
    return {
      primary: "正在下载",
      secondary: version ? `v${version}` : undefined,
      percentLine: `${percent}%`,
    };
  }

  if (version) {
    return { primary: "正在下载", secondary: `v${version}` };
  }

  return { primary: "正在下载" };
}

function UpdateProgressRing({
  percent,
  spinning,
}: {
  percent?: number;
  spinning: boolean;
}) {
  const dashOffset =
    percent !== undefined
      ? RING_CIRCUMFERENCE - (percent / 100) * RING_CIRCUMFERENCE
      : RING_CIRCUMFERENCE * 0.25;

  return (
    <svg
      viewBox="0 0 48 48"
      className={`h-12 w-12 text-accent ${spinning ? "animate-spin" : ""}`}
      aria-hidden="true"
      fill="none"
      stroke="currentColor"
      strokeWidth="3"
    >
      <circle cx="24" cy="24" r={RING_RADIUS} className="opacity-35" />
      <circle
        cx="24"
        cy="24"
        r={RING_RADIUS}
        strokeDasharray={RING_CIRCUMFERENCE}
        strokeDashoffset={dashOffset}
        strokeLinecap="round"
        transform="rotate(-90 24 24)"
      />
    </svg>
  );
}

export function UpdateProgressOverlay() {
  const progress = useUpdateProgress();

  if (progress.phase === "idle") return null;

  const percent =
    progress.phase === "downloading" ? computeUpdatePercent(progress) : undefined;
  const spinning =
    progress.phase === "installing" ||
    (progress.phase === "downloading" && percent === undefined);
  const copy = formatUpdateOverlayCopy(progress.phase, progress.version, percent);

  return (
    <div
      className="fixed inset-0 z-[60] flex items-center justify-center bg-black/45"
      role="dialog"
      aria-modal="true"
      aria-label="正在更新"
      aria-busy="true"
    >
      <div className="config-surface panel flex min-w-[17rem] flex-col items-center gap-2 rounded-lg px-6 py-5 text-center shadow-xl">
        <UpdateProgressRing percent={percent} spinning={spinning} />
        <p className="text-sm font-medium text-fg">{copy.primary}</p>
        {copy.secondary && <p className="text-xs text-fg-secondary">{copy.secondary}</p>}
        {copy.percentLine && (
          <p className="text-xs tabular-nums text-fg-secondary">{copy.percentLine}</p>
        )}
      </div>
    </div>
  );
}
