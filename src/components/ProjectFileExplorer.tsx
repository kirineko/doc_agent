import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { joinPath, parentPath, pathSegments, segmentTarget } from "../lib/pathUtils";
import { ProjectDirListing } from "../types";

interface ProjectFileExplorerProps {
  projectId?: string;
  fileRevision?: number;
}

interface BreadcrumbProps {
  currentPath: string;
  onNavigate: (path: string) => void;
}

interface BreadcrumbSegment {
  label: string;
  path: string | null;
  key: string;
}

const ROOT_BTN_CLASS =
  "inline-flex min-h-6 min-w-6 shrink-0 items-center justify-center rounded text-base leading-none text-slate-400 hover:bg-slate-800/60 hover:text-cyan-300";

function Breadcrumb({ currentPath, onNavigate }: BreadcrumbProps) {
  const segments = pathSegments(currentPath);
  const isRoot = currentPath === ".";

  if (isRoot) {
    return (
      <div className="mb-1 flex items-center" aria-current="page">
        <span
          className={`${ROOT_BTN_CLASS} cursor-default text-slate-200 hover:bg-transparent hover:text-slate-200`}
          aria-label="项目根目录"
          title="项目根目录"
        >
          ⌂
        </span>
      </div>
    );
  }

  const useEllipsis = segments.length > 2;
  const visibleSegments: BreadcrumbSegment[] = useEllipsis
    ? [
        { label: "…", path: null, key: "0-ellipsis" },
        {
          label: segments[segments.length - 2],
          path: segmentTarget(segments, segments.length - 2),
          key: `${segments.length - 2}-${segments[segments.length - 2]}`,
        },
        {
          label: segments[segments.length - 1],
          path: null,
          key: `${segments.length - 1}-${segments[segments.length - 1]}`,
        },
      ]
    : segments.map((seg, index) => ({
        label: seg,
        path: index < segments.length - 1 ? segmentTarget(segments, index) : null,
        key: `${index}-${seg}`,
      }));

  return (
    <div className="mb-1 flex min-w-0 items-center gap-0.5 truncate text-xs">
      <button
        type="button"
        className={ROOT_BTN_CLASS}
        aria-label="项目根目录"
        title="返回项目根目录"
        onClick={() => onNavigate(".")}
      >
        ⌂
      </button>
      {visibleSegments.map((item) => (
        <span key={item.key} className="flex min-w-0 items-center gap-0.5">
          <span className="shrink-0 text-slate-600">/</span>
          {item.path ? (
            <button
              type="button"
              className="truncate text-slate-400 hover:text-cyan-300"
              onClick={() => {
                if (item.path) onNavigate(item.path);
              }}
            >
              {item.label}
            </button>
          ) : (
            <span className="truncate text-slate-200" aria-current="page">
              {item.label}
            </span>
          )}
        </span>
      ))}
    </div>
  );
}

export function ProjectFileExplorer({
  projectId,
  fileRevision = 0,
}: ProjectFileExplorerProps) {
  const [listing, setListing] = useState<ProjectDirListing | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const currentPath = listing?.path ?? ".";
  const loadSeqRef = useRef(0);
  const currentPathRef = useRef(currentPath);
  currentPathRef.current = currentPath;

  const loadDir = useCallback(async (project: string, path: string) => {
    const seq = ++loadSeqRef.current;
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<ProjectDirListing>("list_project_dir_cmd", {
        projectId: project,
        relativePath: path,
      });
      if (seq !== loadSeqRef.current) return;
      setListing(result);
    } catch (e) {
      if (seq !== loadSeqRef.current) return;
      setError(String(e));
    } finally {
      if (seq === loadSeqRef.current) {
        setLoading(false);
      }
    }
  }, []);

  useEffect(() => {
    setListing(null);
    setError(null);
    if (!projectId) return;
    void loadDir(projectId, ".");
  }, [projectId, loadDir]);

  useEffect(() => {
    if (!projectId || fileRevision === 0) return;
    void loadDir(projectId, currentPathRef.current);
  }, [fileRevision, projectId, loadDir]);

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
        {projectId && (
          <button
            type="button"
            className="inline-flex min-h-6 min-w-6 items-center justify-center rounded text-xs text-slate-400 hover:bg-slate-800/80 hover:text-cyan-300 disabled:cursor-not-allowed disabled:opacity-40"
            aria-label="刷新当前目录"
            title="刷新当前目录"
            disabled={loading}
            onClick={() => void loadDir(projectId, currentPath)}
          >
            ↻
          </button>
        )}
      </div>
      {projectId ? (
        <Breadcrumb
          currentPath={currentPath}
          onNavigate={(path) => void loadDir(projectId, path)}
        />
      ) : (
        <div className="mb-1 truncate text-[10px] text-slate-500">未选择项目</div>
      )}
      <div className="min-h-0 flex-1 space-y-0.5 overflow-y-auto">
        {!projectId && (
          <div className="rounded-md border border-dashed border-slate-700 p-2 text-[11px] text-slate-500">
            选择项目后可浏览文件。
          </div>
        )}
        {loading && <div className="text-[11px] text-slate-500">加载中…</div>}
        {error && <div className="text-[11px] text-rose-400">{error}</div>}
        {projectId && currentPath !== "." && (
          <button
            type="button"
            disabled={loading}
            className="flex w-full items-center gap-1.5 rounded border border-dashed border-slate-700/80 bg-slate-900/40 px-1 py-0.5 text-left text-[11px] text-slate-400 hover:border-slate-600 hover:text-cyan-300 disabled:cursor-not-allowed disabled:opacity-40"
            onClick={() => void loadDir(projectId, parentPath(currentPath))}
          >
            <span className="shrink-0">📂</span>
            <span>返回上级</span>
          </button>
        )}
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
