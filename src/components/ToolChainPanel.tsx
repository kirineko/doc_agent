import { useEffect, useLayoutEffect, useRef } from "react";
import { formatToolResultError } from "../lib/fileBusy";
import { formatToolArgs, toolLabel } from "../lib/toolLabels";
import { PanelSectionHeader } from "./PanelSectionHeader";

export interface LiveToolCall {
  id: string;
  name: string;
  args: unknown;
  status: string;
  argsChars?: number;
  summary?: string;
}

interface ToolChainPanelProps {
  items: LiveToolCall[];
  collapsed?: boolean;
  onToggleCollapse?: () => void;
}

function statusLabel(status: string): string {
  switch (status) {
    case "streaming":
      return "生成参数中";
    case "running":
      return "执行中";
    case "awaiting_user":
      return "等待回答";
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
    case "awaiting_user":
      return "text-sky-500";
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

export function ToolChainPanel({
  items,
  collapsed = false,
  onToggleCollapse,
}: ToolChainPanelProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const bottomRef = useRef<HTMLDivElement>(null);
  const stickToBottomRef = useRef(true);

  const scrollToBottom = () => {
    bottomRef.current?.scrollIntoView({ behavior: "auto", block: "end" });
  };

  const handleScroll = () => {
    const el = scrollRef.current;
    if (!el) return;
    stickToBottomRef.current = el.scrollHeight - el.scrollTop - el.clientHeight < 72;
  };

  useEffect(() => {
    if (items.length === 0) stickToBottomRef.current = true;
  }, [items.length]);

  useLayoutEffect(() => {
    if (!stickToBottomRef.current || items.length === 0) return;
    scrollToBottom();
  }, [items.length]);

  return (
    <section className="flex h-full min-h-0 flex-col">
      {onToggleCollapse ? (
        <PanelSectionHeader
          title="工具调用链"
          collapsed={collapsed}
          onToggleCollapse={onToggleCollapse}
        />
      ) : (
        <div className="mb-1.5 shrink-0 text-xs font-medium text-fg-heading">工具调用链</div>
      )}
      {!collapsed && (
      <div
        ref={scrollRef}
        onScroll={handleScroll}
        className="min-h-0 flex-1 space-y-1.5 overflow-y-auto"
      >
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
            {item.status === "error" && item.summary ? (
              <details className="mt-1">
                <summary className="cursor-pointer text-[10px] text-rose-600 hover:text-rose-700 dark:text-rose-400 dark:hover:text-rose-300">
                  错误详情
                </summary>
                <div
                  className="mt-1 rounded border border-rose-500/25 bg-rose-500/10 px-1.5 py-1 text-[10px] leading-4 text-rose-700 dark:text-rose-300"
                  role="alert"
                >
                  {formatToolResultError(item.summary)}
                </div>
              </details>
            ) : null}
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
        <div ref={bottomRef} className="h-px shrink-0" aria-hidden />
      </div>
      )}
    </section>
  );
}
