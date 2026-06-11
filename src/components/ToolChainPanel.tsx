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
      return "animate-pulse text-sky-300";
    case "running":
      return "text-amber-300";
    case "done":
      return "text-emerald-300";
    case "error":
      return "text-rose-300";
    default:
      return "text-slate-400";
  }
}

export function formatCharCount(count: number): string {
  if (count < 1000) return `${count} 字符`;
  return `${(count / 1000).toFixed(1)}K 字符`;
}

export function ToolChainPanel({ items }: ToolChainPanelProps) {
  return (
    <section className="flex min-h-0 flex-1 flex-col">
      <div className="mb-1.5 text-xs font-medium text-slate-200">工具调用链</div>
      <div className="min-h-0 flex-1 space-y-1.5 overflow-y-auto">
        {items.length === 0 && (
          <div className="rounded-md border border-dashed border-slate-700 p-2.5 text-[11px] text-slate-500">
            工具调用会在这里实时显示。
          </div>
        )}
        {items.map((item, index) => (
          <div key={item.id} className="rounded-md border border-slate-800 bg-slate-950/50 px-2 py-1.5">
            <div className="flex items-center justify-between gap-2">
              <div className="min-w-0 truncate text-xs font-medium text-cyan-200">
                {index + 1}. {toolLabel(item.name)}
              </div>
              <div className={`shrink-0 text-[10px] ${statusClass(item.status)}`}>
                {statusLabel(item.status)}
              </div>
            </div>
            {item.status === "streaming" ? (
              <div className="mt-1 text-[10px] text-slate-400">
                正在接收参数… 已收到 {formatCharCount(item.argsChars ?? 0)}
              </div>
            ) : (
              <details className="mt-1">
                <summary className="cursor-pointer text-[10px] text-slate-500 hover:text-slate-300">
                  参数
                </summary>
                <pre className="mt-1 max-h-24 overflow-auto rounded bg-black/30 p-1.5 text-[10px] leading-4 text-slate-300">
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
