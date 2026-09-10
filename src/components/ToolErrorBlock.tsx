import { isFileBusySummary, parseToolResultError } from "../lib/fileBusy";

export function ToolErrorBlock({ summary }: { summary: string }) {
  if (isFileBusySummary(summary)) {
    const { headline } = parseToolResultError(summary);
    return (
      <details className="mt-1">
        <summary className="cursor-pointer text-[10px] text-rose-600 hover:text-rose-700 dark:text-rose-400 dark:hover:text-rose-300">
          错误详情
        </summary>
        <div
          className="mt-1 rounded border border-rose-500/25 bg-rose-500/10 px-1.5 py-1 text-[10px] leading-4 text-rose-700 dark:text-rose-300"
          role="alert"
        >
          {headline}
        </div>
      </details>
    );
  }

  const parsed = parseToolResultError(summary);
  const hasFold = Boolean(parsed.detail || parsed.hint);

  return (
    <div className="mt-1" role="alert">
      <div className="text-[10px] font-medium text-rose-600 dark:text-rose-400">
        {parsed.headline}
      </div>
      {hasFold ? (
        <details className="mt-1">
          <summary className="cursor-pointer text-[10px] text-rose-600 hover:text-rose-700 dark:text-rose-400 dark:hover:text-rose-300">
            详情
          </summary>
          <div className="mt-1 rounded border border-rose-500/25 bg-rose-500/10 px-1.5 py-1 text-[10px] leading-4 text-rose-700 dark:text-rose-300">
            {parsed.detail ? (
              <pre className="whitespace-pre-wrap break-all font-sans">{parsed.detail}</pre>
            ) : null}
            {parsed.hint ? <p className="mt-1 text-fg-secondary">{parsed.hint}</p> : null}
          </div>
        </details>
      ) : null}
    </div>
  );
}
