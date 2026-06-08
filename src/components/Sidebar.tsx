import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { formatSessionTime } from "../lib/formatTime";
import { MODEL_OPTIONS, Project, Session, providerLabel } from "../types";

interface SidebarProps {
  projects: Project[];
  sessions: Session[];
  activeProjectId?: string;
  activeSessionId?: string;
  apiKeyStatus: Record<string, boolean>;
  onProjectsChange: (projects: Project[]) => void;
  onSessionsChange: (sessions: Session[]) => void;
  onSelectProject: (projectId: string) => void;
  onSelectSession: (sessionId: string | undefined) => void;
  onSessionUpdated: (session: Session) => void;
  onApiKeyStatusChange: (provider: string, has: boolean) => void;
}

export function Sidebar({
  projects,
  sessions,
  activeProjectId,
  activeSessionId,
  apiKeyStatus,
  onProjectsChange,
  onSessionsChange,
  onSelectProject,
  onSelectSession,
  onSessionUpdated,
  onApiKeyStatusChange,
}: SidebarProps) {
  const activeSession = sessions.find((s) => s.id === activeSessionId);
  const activeModel = MODEL_OPTIONS.find((m) => m.id === activeSession?.model);
  const activeProvider = activeModel?.provider ?? null;
  const [apiKeyInput, setApiKeyInput] = useState("");
  const [keyError, setKeyError] = useState<string>();
  const [showKeyReplace, setShowKeyReplace] = useState(false);

  useEffect(() => {
    setApiKeyInput("");
    setKeyError(undefined);
    setShowKeyReplace(false);
  }, [activeProvider]);

  async function pickProject() {
    const selected = await open({ directory: true, multiple: false });
    if (!selected || Array.isArray(selected)) return;
    const name = selected.split(/[/\\]/).pop() || "project";
    const project = await invoke<Project>("create_project", {
      req: { name, root_path: selected },
    });
    onProjectsChange([project, ...projects]);
    onSelectProject(project.id);
  }

  async function createSession() {
    if (!activeProjectId) return;
    const session = await invoke<Session>("create_session", {
      req: {
        project_id: activeProjectId,
        title: "新会话",
        model: "deepseek-v4-flash",
      },
    });
    onSessionsChange([session, ...sessions]);
    onSelectSession(session.id);
  }

  async function updateSession(patch: Partial<Session>) {
    if (!activeSession) return;
    const updated = await invoke<Session>("update_session", {
      req: {
        session_id: activeSession.id,
        title: patch.title,
        model: patch.model,
        thinking_enabled: patch.thinking_enabled,
        thinking_effort: patch.thinking_effort,
      },
    });
    onSessionUpdated(updated);
  }

  async function saveApiKey() {
    if (!activeProvider) return;
    const value = apiKeyInput.trim();
    if (!value) {
      setKeyError("请输入 API Key");
      return;
    }
    try {
      await invoke("set_api_key", { provider: activeProvider, apiKey: value });
      const has = await invoke<boolean>("has_api_key", { provider: activeProvider });
      onApiKeyStatusChange(activeProvider, has);
      setApiKeyInput("");
      setKeyError(undefined);
      setShowKeyReplace(false);
    } catch (error) {
      setKeyError(String(error));
    }
  }

  async function clearApiKey() {
    if (!activeProvider) return;
    try {
      await invoke("clear_api_key", { provider: activeProvider });
      const has = await invoke<boolean>("has_api_key", { provider: activeProvider });
      onApiKeyStatusChange(activeProvider, has);
      setApiKeyInput("");
      setKeyError(undefined);
      setShowKeyReplace(false);
    } catch (error) {
      setKeyError(String(error));
    }
  }

  async function deleteSession(sessionId: string) {
    try {
      await invoke("delete_session", { sessionId });
      const next = sessions.filter((item) => item.id !== sessionId);
      onSessionsChange(next);
      if (activeSessionId === sessionId) {
        onSelectSession(next[0]?.id);
      }
    } catch (error) {
      console.error(error);
    }
  }

  return (
    <aside className="panel flex h-full w-72 shrink-0 flex-col gap-2.5 p-3">
      <div className="shrink-0">
        <div className="mb-1 text-[11px] uppercase tracking-[0.16em] text-slate-400">项目</div>
        <button
          className="mb-2 w-full rounded-md bg-indigo-600 px-2.5 py-1.5 text-xs font-medium hover:bg-indigo-500"
          onClick={pickProject}
        >
          选择目录创建项目
        </button>
        <div className="max-h-28 space-y-1 overflow-y-auto">
          {projects.map((project) => (
            <button
              key={project.id}
              className={`w-full rounded-md border px-2.5 py-1.5 text-left text-xs ${
                project.id === activeProjectId
                  ? "border-indigo-500 bg-indigo-950/40"
                  : "border-slate-800 hover:border-slate-600"
              }`}
              onClick={() => onSelectProject(project.id)}
            >
              <div className="font-medium">{project.name}</div>
              <div className="truncate text-[11px] text-slate-400">{project.root_path}</div>
            </button>
          ))}
        </div>
      </div>

      <div className="flex min-h-0 flex-1 flex-col">
        <div className="mb-1 flex shrink-0 items-center justify-between">
          <div className="text-[11px] uppercase tracking-[0.16em] text-slate-400">会话</div>
          <button
            className="rounded border border-slate-700 px-1.5 py-0.5 text-[11px] hover:border-slate-500"
            onClick={createSession}
            disabled={!activeProjectId}
          >
            新建
          </button>
        </div>
        <div className="min-h-0 flex-1 space-y-1 overflow-y-auto">
          {sessions.map((session) => (
            <div
              key={session.id}
              className={`group flex items-stretch rounded-md border text-xs ${
                session.id === activeSessionId
                  ? "border-cyan-500 bg-cyan-950/30"
                  : "border-slate-800 hover:border-slate-600"
              }`}
            >
              <button
                className="min-w-0 flex-1 px-2.5 py-1.5 text-left"
                onClick={() => onSelectSession(session.id)}
              >
                <div className="truncate font-medium">{session.title}</div>
                <div className="text-[11px] text-slate-400">{formatSessionTime(session.updated_at)}</div>
              </button>
              <button
                type="button"
                className="shrink-0 border-l border-transparent px-2 text-slate-500 opacity-0 transition hover:text-rose-400 group-hover:border-slate-700 group-hover:opacity-100"
                title="删除会话"
                onClick={() => void deleteSession(session.id)}
              >
                ×
              </button>
            </div>
          ))}
        </div>
      </div>

      {activeSession && (
        <div className="shrink-0 space-y-2 border-t border-slate-800 pt-2.5">
          <div className="text-[11px] uppercase tracking-[0.16em] text-slate-400">模型配置</div>
          <select
            className="w-full rounded-md border border-slate-700 bg-slate-900 px-2.5 py-1.5 text-xs"
            value={activeSession.model}
            onChange={(e) => updateSession({ model: e.target.value })}
          >
            {MODEL_OPTIONS.map((model) => (
              <option key={model.id} value={model.id}>
                {model.label}
              </option>
            ))}
          </select>

          <label className="flex items-center gap-2 text-xs">
            <input
              type="checkbox"
              checked={activeSession.thinking_enabled}
              onChange={(e) => updateSession({ thinking_enabled: e.target.checked })}
            />
            启用思考模式
          </label>

          {activeModel?.supportsEffort && activeSession.thinking_enabled && (
            <select
              className="w-full rounded-md border border-slate-700 bg-slate-900 px-2.5 py-1.5 text-xs"
              value={activeSession.thinking_effort}
              onChange={(e) => updateSession({ thinking_effort: e.target.value })}
            >
              <option value="high">high</option>
              <option value="max">max</option>
            </select>
          )}

          {activeProvider &&
            (apiKeyStatus[activeProvider] ? (
              <details className="rounded-md border border-slate-800 bg-slate-950/40 p-1.5">
                <summary className="flex cursor-pointer list-none items-center justify-between text-[11px] marker:content-none [&::-webkit-details-marker]:hidden">
                  <span className="text-slate-400">{providerLabel(activeProvider)} API Key</span>
                  <span className="text-emerald-400">已保存</span>
                </summary>
                <div className="mt-1 space-y-1">
                  <div className="flex items-center justify-end gap-1.5">
                    <button
                      type="button"
                      className="rounded border border-slate-700 px-1.5 py-0.5 text-[11px] text-slate-400 hover:border-slate-500 hover:text-slate-200"
                      onClick={() => setShowKeyReplace((value) => !value)}
                    >
                      更换 Key
                    </button>
                    <button
                      type="button"
                      className="rounded border border-slate-700 px-1.5 py-0.5 text-[11px] text-slate-400 hover:border-rose-500 hover:text-rose-300"
                      onClick={() => void clearApiKey()}
                    >
                      清空
                    </button>
                  </div>
                  {showKeyReplace && (
                    <div className="space-y-1">
                      <input
                        type="password"
                        className="w-full rounded-md border border-slate-700 bg-slate-900 px-2 py-1 text-xs outline-none focus:border-indigo-500"
                        placeholder="输入新 Key 可覆盖保存"
                        value={apiKeyInput}
                        onChange={(e) => {
                          setApiKeyInput(e.target.value);
                          setKeyError(undefined);
                        }}
                        onKeyDown={(e) => {
                          if (e.key === "Enter") void saveApiKey();
                        }}
                      />
                      <button
                        type="button"
                        className="w-full rounded-md border border-indigo-700 bg-indigo-950/40 px-2 py-0.5 text-[11px] hover:border-indigo-500"
                        onClick={() => void saveApiKey()}
                      >
                        保存
                      </button>
                    </div>
                  )}
                  {keyError && <div className="text-[11px] text-rose-400">{keyError}</div>}
                </div>
              </details>
            ) : (
              <div className="rounded-md border border-slate-800 bg-slate-950/40 p-1.5">
                <div className="flex items-center justify-between text-[11px]">
                  <span className="text-slate-400">{providerLabel(activeProvider)} API Key</span>
                  <span className="text-amber-400">未配置</span>
                </div>
                <div className="mt-1 space-y-1">
                  <input
                    type="password"
                    className="w-full rounded-md border border-slate-700 bg-slate-900 px-2 py-1 text-xs outline-none focus:border-indigo-500"
                    placeholder="输入 API Key"
                    value={apiKeyInput}
                    onChange={(e) => {
                      setApiKeyInput(e.target.value);
                      setKeyError(undefined);
                    }}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") void saveApiKey();
                    }}
                  />
                  <button
                    type="button"
                    className="w-full rounded-md border border-indigo-700 bg-indigo-950/40 px-2 py-0.5 text-[11px] hover:border-indigo-500"
                    onClick={() => void saveApiKey()}
                  >
                    保存
                  </button>
                  {keyError && <div className="text-[11px] text-rose-400">{keyError}</div>}
                </div>
              </div>
            ))}
        </div>
      )}
    </aside>
  );
}
