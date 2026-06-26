import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { joinPath, parentPath, pathSegments, segmentTarget } from "../lib/pathUtils";
import { ProjectDirListing } from "../types";
import { FolderIcon, FolderOpenIcon, FileIcon, panelIconButtonClassName, RefreshIcon } from "./PanelIcons";

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
  "inline-flex h-6 w-6 shrink-0 items-center justify-center rounded text-xs leading-none text-fg-secondary hover:bg-hover hover:text-link";

const FILE_ROW_ICON_CLASS = "inline-flex h-4 w-4 shrink-0 items-center justify-center text-fg-muted";

function Breadcrumb({ currentPath, onNavigate }: BreadcrumbProps) {
  const segments = pathSegments(currentPath);
  const isRoot = currentPath === ".";

  if (isRoot) {
    return (
      <div className="mb-1 flex items-center" aria-current="page">
        <span
          className={`${ROOT_BTN_CLASS} cursor-default text-fg-heading hover:bg-transparent hover:text-fg-heading`}
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
          <span className="shrink-0 text-fg-muted">/</span>
          {item.path ? (
            <button
              type="button"
              className="truncate text-fg-secondary hover:text-link"
              onClick={() => {
                if (item.path) onNavigate(item.path);
              }}
            >
              {item.label}
            </button>
          ) : (
            <span className="truncate text-fg-heading" aria-current="page">
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

  async function openProjectRoot() {
    if (!projectId) return;
    try {
      await invoke("open_project_root", { projectId });
    } catch (e) {
      setError(String(e));
    }
  }

  const panelIconBtn = panelIconButtonClassName();
  const toolbarActions = projectId ? (
    <>
      <button
        type="button"
        className={panelIconBtn}
        aria-label="在文件管理器中打开项目根目录"
        title="在文件管理器中打开项目根目录"
        disabled={loading}
        onClick={() => void openProjectRoot()}
      >
        <FolderOpenIcon className="h-3.5 w-3.5" />
      </button>
      <button
        type="button"
        className={panelIconBtn}
        aria-label="刷新当前目录"
        title="刷新当前目录"
        disabled={loading}
        onClick={() => void loadDir(projectId, currentPath)}
      >
        <RefreshIcon className="h-3.5 w-3.5" />
      </button>
    </>
  ) : null;

  return (
    <div className="flex h-full min-h-0 flex-col">
      {projectId ? (
        <div className="mb-1 flex min-w-0 items-center gap-1">
          <div className="min-w-0 flex-1">
            <Breadcrumb
              currentPath={currentPath}
              onNavigate={(path) => void loadDir(projectId, path)}
            />
          </div>
          <div className="flex shrink-0 items-center gap-0.5">{toolbarActions}</div>
        </div>
      ) : (
        <div className="mb-1 truncate text-[10px] leading-none text-fg-muted">未选择项目</div>
      )}
      <div className="min-h-0 flex-1 space-y-0.5 overflow-y-auto">
        {!projectId && (
          <div className="rounded-md border border-dashed border-border-subtle p-2 text-[11px] text-fg-muted">
            选择项目后可浏览文件。
          </div>
        )}
        {loading && <div className="text-[11px] text-fg-muted">加载中…</div>}
        {error && <div className="text-[11px] text-rose-500">{error}</div>}
        {projectId && currentPath !== "." && (
          <button
            type="button"
            disabled={loading}
            className="chip-surface flex w-full items-center gap-1.5 rounded border-dashed px-1 py-0.5 text-left text-[11px] leading-none disabled:cursor-not-allowed disabled:opacity-40"
            onClick={() => void loadDir(projectId, parentPath(currentPath))}
          >
            <span className={FILE_ROW_ICON_CLASS}>
              <FolderOpenIcon className="h-3 w-3" />
            </span>
            <span className="truncate">返回上级</span>
          </button>
        )}
        {listing?.entries.map((entry) => (
          <button
            key={entry.name}
            type="button"
            title={entry.is_dir ? "点击进入" : "双击用默认应用打开"}
            className="flex w-full items-center gap-1.5 rounded px-1 py-0.5 text-left text-[11px] leading-none text-fg hover:bg-hover"
            onClick={() => {
              if (entry.is_dir && projectId) {
                void loadDir(projectId, joinPath(currentPath, entry.name));
              }
            }}
            onDoubleClick={() => {
              if (!entry.is_dir) void openFile(entry.name);
            }}
          >
            <span className={FILE_ROW_ICON_CLASS}>
              {entry.is_dir ? (
                <FolderIcon className="h-3 w-3" />
              ) : (
                <FileIcon className="h-3 w-3" />
              )}
            </span>
            <span className="truncate">{entry.name}</span>
          </button>
        ))}
        {projectId && listing && listing.entries.length === 0 && !loading && (
          <div className="text-[11px] text-fg-muted">空目录</div>
        )}
      </div>
    </div>
  );
}
