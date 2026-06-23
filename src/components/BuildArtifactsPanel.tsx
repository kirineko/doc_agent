import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { TurnArtifact } from "../lib/agentEvents";

interface BuildArtifactsPanelProps {
  artifacts: TurnArtifact[];
  projectId?: string;
  collapsed?: boolean;
}

/** 按扩展名给出文件 emoji；无扩展名（如目录）统一 📄。
 *  open/reveal 对文件与目录均有效，故面板不做目录区分。 */
function artifactIcon(path: string): string {
  const ext = path.split(".").pop()?.toLowerCase() ?? "";
  if (["docx", "doc"].includes(ext)) return "📄";
  if (["xlsx", "xls"].includes(ext)) return "📊";
  if (["pptx", "ppt"].includes(ext)) return "📽️";
  if (ext === "pdf") return "📕";
  if (["md", "txt", "csv", "json", "html", "htm"].includes(ext)) return "📃";
  return "📄";
}

function baseName(path: string): string {
  const trimmed = path.replace(/\/$/, "");
  const idx = trimmed.lastIndexOf("/");
  return idx >= 0 ? trimmed.slice(idx + 1) : trimmed;
}

export function BuildArtifactsPanel({
  artifacts,
  projectId,
  collapsed = false,
}: BuildArtifactsPanelProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const bottomRef = useRef<HTMLDivElement>(null);
  const stickToBottomRef = useRef(true);
  const [error, setError] = useState<string | null>(null);

  const scrollToBottom = () => {
    bottomRef.current?.scrollIntoView({ behavior: "auto", block: "end" });
  };

  const handleScroll = () => {
    const el = scrollRef.current;
    if (!el) return;
    stickToBottomRef.current = el.scrollHeight - el.scrollTop - el.clientHeight < 72;
  };

  useEffect(() => {
    if (artifacts.length === 0) stickToBottomRef.current = true;
  }, [artifacts.length]);

  useLayoutEffect(() => {
    if (!stickToBottomRef.current || artifacts.length === 0) return;
    scrollToBottom();
  }, [artifacts.length]);

  async function openArtifact(path: string) {
    if (!projectId) return;
    setError(null);
    try {
      await invoke("open_project_file", { projectId, relativePath: path });
    } catch (e) {
      setError(String(e));
    }
  }

  async function revealArtifact(path: string) {
    if (!projectId) return;
    setError(null);
    try {
      await invoke("reveal_project_file", { projectId, relativePath: path });
    } catch (e) {
      setError(String(e));
    }
  }

  if (collapsed) return null;

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div
        ref={scrollRef}
        onScroll={handleScroll}
        className="min-h-0 flex-1 space-y-1.5 overflow-y-auto"
      >
        {artifacts.length === 0 && (
          <div className="rounded-md border border-dashed border-border-subtle p-2.5 text-[11px] text-fg-muted">
            本轮没有产生或修改文件。
          </div>
        )}
        {error && (
          <div className="rounded border border-rose-500/25 bg-rose-500/10 px-1.5 py-1 text-[10px] text-rose-600 dark:text-rose-300">
            {error}
          </div>
        )}
        {artifacts.map((item) => (
          <div key={item.path} className="tool-item-surface rounded-md px-2 py-1.5">
            <div className="flex min-w-0 items-center gap-1.5">
              <span className="shrink-0 text-fg-muted">{artifactIcon(item.path)}</span>
              <div className="min-w-0 flex-1">
                <div className="truncate text-xs font-medium text-link" title={item.path}>
                  {baseName(item.path)}
                </div>
                <div className="truncate text-[10px] text-fg-muted" title={item.path}>
                  {item.path}
                </div>
              </div>
            </div>
            <div className="mt-1 flex items-center justify-between gap-2">
              <span className="shrink-0 text-[10px] text-fg-secondary">
                ↳ {item.sourceToolLabel}
              </span>
              <div className="flex shrink-0 items-center gap-1.5">
                <button
                  type="button"
                  className="rounded px-1 text-[10px] text-fg-secondary hover:bg-hover hover:text-link"
                  title="用默认程序打开"
                  onClick={() => void openArtifact(item.path)}
                >
                  打开
                </button>
                <button
                  type="button"
                  className="rounded px-1 text-[10px] text-fg-secondary hover:bg-hover hover:text-link"
                  title="在文件夹中显示"
                  onClick={() => void revealArtifact(item.path)}
                >
                  定位
                </button>
              </div>
            </div>
          </div>
        ))}
        <div ref={bottomRef} className="h-px shrink-0" aria-hidden />
      </div>
    </div>
  );
}
