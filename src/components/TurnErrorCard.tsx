import type { RetryNotice, TurnError } from "../types";

export function retryNoticeText(notice: RetryNotice): string {
  return `网络波动，正在重试（${notice.attempt}/${notice.max}）…`;
}

export function TurnErrorCard({ error }: { error: TurnError }) {
  async function copyDetail() {
    if (!error.detail) return;
    await navigator.clipboard.writeText(error.detail);
  }

  return (
    <div
      className="rounded-md border border-rose-500/30 bg-rose-500/10 px-3 py-2 text-sm text-rose-800 dark:text-rose-200"
      role="alert"
    >
      <p className="font-medium">{error.message}</p>
      {error.hint ? <p className="mt-1 text-xs text-fg-secondary">{error.hint}</p> : null}
      {error.detail ? (
        <details className="mt-2">
          <summary className="cursor-pointer text-xs text-rose-700 hover:text-rose-800 dark:text-rose-300">
            详情
          </summary>
          <pre className="mt-1 max-h-40 overflow-auto whitespace-pre-wrap break-all font-sans text-[11px] leading-4">
            {error.detail}
          </pre>
          <button
            type="button"
            className="mt-1 text-xs text-rose-700 underline dark:text-rose-300"
            onClick={() => void copyDetail()}
          >
            复制详情
          </button>
        </details>
      ) : null}
    </div>
  );
}
