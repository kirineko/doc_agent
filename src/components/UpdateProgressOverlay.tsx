import { computeUpdatePercent } from "../lib/updateProgress";
import { useUpdateProgress } from "../hooks/useUpdateProgress";

const RING_RADIUS = 20;
const RING_CIRCUMFERENCE = 2 * Math.PI * RING_RADIUS;

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

function formatUpdateMessage(
  phase: "downloading" | "installing",
  version?: string,
  percent?: number,
): string {
  if (phase === "installing") {
    return "正在安装，即将重启…";
  }

  if (percent !== undefined) {
    const versionLabel = version ? ` v${version}` : "";
    return `正在下载${versionLabel}… ${percent}%`;
  }

  if (version) {
    return `正在下载 v${version}…`;
  }

  return "正在下载更新…";
}

export function UpdateProgressOverlay() {
  const progress = useUpdateProgress();

  if (progress.phase === "idle") return null;

  const percent =
    progress.phase === "downloading" ? computeUpdatePercent(progress) : undefined;
  const spinning =
    progress.phase === "installing" ||
    (progress.phase === "downloading" && percent === undefined);
  const message = formatUpdateMessage(progress.phase, progress.version, percent);

  return (
    <div
      className="fixed inset-0 z-[60] flex items-center justify-center bg-black/45"
      role="dialog"
      aria-modal="true"
      aria-label="正在更新"
      aria-busy="true"
    >
      <div className="config-surface panel flex flex-col items-center gap-3 rounded-lg px-6 py-5 shadow-xl">
        <UpdateProgressRing percent={percent} spinning={spinning} />
        <p className="text-sm text-fg">{message}</p>
      </div>
    </div>
  );
}
