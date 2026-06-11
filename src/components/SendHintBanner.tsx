import { providerLabel } from "../types";
import type { SendBlocker } from "../lib/sendReadiness";

interface SendHintBannerProps {
  blocker: SendBlocker;
  onDismiss: () => void;
}

export function SendHintBanner({ blocker, onDismiss }: SendHintBannerProps) {
  const message =
    blocker.kind === "no_project"
      ? "请先选择或创建项目目录"
      : `请先配置 ${providerLabel(blocker.provider)} API Key`;

  return (
    <div
      role="alert"
      className="flex items-center justify-between gap-2 rounded-md border border-amber-700/60 bg-amber-950/40 px-3 py-2 text-xs text-amber-200"
    >
      <span>{message}</span>
      <button
        type="button"
        className="shrink-0 text-amber-400/80 hover:text-amber-200"
        onClick={onDismiss}
        aria-label="关闭"
      >
        ×
      </button>
    </div>
  );
}
