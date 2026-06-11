import { formatToolArgs, toolLabel } from "../lib/toolLabels";

export interface LiveToolCall {
  id: string;
  name: string;
  args: unknown;
  status: string;
  argsChars?: number;
}

interface ToolChainPanelProps {
  items: LiveToolCall[];
}

function statusLabel(status: string): string {
  switch (status) {
    case "streaming":
      return "生成参数中";
    case "running":
      return "执行中";
    case "done":
      return "完成";
    case "error":
      return "失败";
    default:
      return status;
  }
}

function statusClass(status: string): string {
  switch (status) {
    case "streaming":
      return "animate-pulse text-sky-500";
    case "running":
      return "text-amber-600";
    case "done":
      return "text-emerald-600";
    case "error":
      return "text-rose-500";
    default:
      return "text-fg-secondary";
  }
}

export function formatCharCount(count: number): string {
  if (count < 1000) return `${count} 字符`;
  return `${(count / 1000).toFixed(1)}K 字符`;
}

export function ToolChainPanel({ items }: ToolChainPanelProps) {
  return (
    <section className="flex min-h-0 flex-1 flex-col">
      <div className="mb-1.5 text-xs font-medium text-fg-heading">工具调用链</div>
      <div className="min-h-0 flex-1 space-y-1.5 overflow-y-auto">
        {items.length === 0 && (
          <div className="rounded-md border border-dashed border-border-subtle p-2.5 text-[11px] text-fg-muted">
            工具调用会在这里实时显示。
          </div>
        )}
        {items.map((item, index) => (
          <div key={item.id} className="tool-item-surface rounded-md px-2 py-1.5">
            <div className="flex items-center justify-between gap-2">
              <div className="min-w-0 truncate text-xs font-medium text-link">
                {index + 1}. {toolLabel(item.name)}
              </div>
              <div className={`shrink-0 text-[10px] ${statusClass(item.status)}`}>
                {statusLabel(item.status)}
              </div>
            </div>
            {item.status === "streaming" ? (
              <div className="mt-1 text-[10px] text-fg-secondary">
                正在接收参数… 已收到 {formatCharCount(item.argsChars ?? 0)}
              </div>
            ) : (
              <details className="mt-1">
                <summary className="cursor-pointer text-[10px] text-fg-muted hover:text-fg-secondary">
                  参数
                </summary>
                <pre className="tool-pre mt-1 max-h-24 overflow-auto rounded p-1.5 text-[10px] leading-4">
                  {formatToolArgs(item.args)}
                </pre>
              </details>
            )}
          </div>
        ))}
      </div>
    </section>
  );
}
