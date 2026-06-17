import { providerLabel } from "../types";
import { PARALLEL_LIMIT_MESSAGE } from "../lib/sessionRunState";
import type { SendBlocker } from "../lib/sendReadiness";

interface SendHintBannerProps {
  blocker: SendBlocker;
  onDismiss: () => void;
}

export function SendHintBanner({ blocker, onDismiss }: SendHintBannerProps) {
  const message =
    blocker.kind === "no_project"
      ? "请先选择或创建项目目录"
      : blocker.kind === "parallel_limit"
        ? PARALLEL_LIMIT_MESSAGE
        : `请先配置 ${providerLabel(blocker.provider)} API Key`;

  return (
    <div
      role="alert"
      className="alert-banner flex items-center justify-between gap-2 rounded-md border px-3 py-2 text-xs"
    >
      <span>{message}</span>
      <button
        type="button"
        className="shrink-0 opacity-80 hover:opacity-100"
        onClick={onDismiss}
        aria-label="关闭"
      >
        ×
      </button>
    </div>
  );
}
