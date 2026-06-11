import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ProjectDirListing } from "../types";

interface ProjectFileExplorerProps {
  projectId?: string;
}

function joinPath(base: string, name: string): string {
  return base === "." ? name : `${base}/${name}`;
}

function parentPath(current: string): string {
  const idx = current.lastIndexOf("/");
  return idx < 0 ? "." : current.slice(0, idx);
}

export function ProjectFileExplorer({ projectId }: ProjectFileExplorerProps) {
  const [listing, setListing] = useState<ProjectDirListing | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const currentPath = listing?.path ?? ".";

  const loadDir = useCallback(async (project: string, path: string) => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<ProjectDirListing>("list_project_dir_cmd", {
        projectId: project,
        relativePath: path,
      });
      setListing(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    setListing(null);
    setError(null);
    if (!projectId) return;
    void loadDir(projectId, ".");
  }, [projectId, loadDir]);

  async function openFile(name: string) {
    if (!projectId) return;
    try {
      await invoke("open_project_file", {
        projectId,
        relativePath: joinPath(currentPath, name),
      });
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <div className="flex min-h-0 flex-[0.38] flex-col border-t border-slate-800 pt-2">
      <div className="mb-1 flex items-center justify-between gap-2">
        <div className="text-xs font-medium text-slate-200">项目文件</div>
        {currentPath !== "." && projectId && (
          <button
            type="button"
            className="text-[10px] text-slate-500 hover:text-cyan-300"
            onClick={() => void loadDir(projectId, parentPath(currentPath))}
          >
            ..
          </button>
        )}
      </div>
      <div className="mb-1 truncate text-[10px] text-slate-500">
        {projectId ? (currentPath === "." ? "/" : `/${currentPath}`) : "未选择项目"}
      </div>
      <div className="min-h-0 flex-1 space-y-0.5 overflow-y-auto">
        {!projectId && (
          <div className="rounded-md border border-dashed border-slate-700 p-2 text-[11px] text-slate-500">
            选择项目后可浏览文件。
          </div>
        )}
        {loading && <div className="text-[11px] text-slate-500">加载中…</div>}
        {error && <div className="text-[11px] text-rose-400">{error}</div>}
        {listing?.entries.map((entry) => (
          <button
            key={entry.name}
            type="button"
            title={entry.is_dir ? "点击进入" : "双击用默认应用打开"}
            className="flex w-full items-center gap-1.5 rounded px-1 py-0.5 text-left text-[11px] text-slate-300 hover:bg-slate-800/80"
            onClick={() => {
              if (entry.is_dir && projectId) {
                void loadDir(projectId, joinPath(currentPath, entry.name));
              }
            }}
            onDoubleClick={() => {
              if (!entry.is_dir) void openFile(entry.name);
            }}
          >
            <span className="shrink-0 text-slate-500">{entry.is_dir ? "📁" : "📄"}</span>
            <span className="truncate">{entry.name}</span>
          </button>
        ))}
        {projectId && listing && listing.entries.length === 0 && !loading && (
          <div className="text-[11px] text-slate-500">空目录</div>
        )}
      </div>
    </div>
  );
}
