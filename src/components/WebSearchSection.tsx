import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

const TAVILY_PROVIDER = "tavily";

interface WebSearchSectionProps {
  enabled: boolean;
  onStatusChange: (has: boolean) => void;
}

function KeyEntry({
  value,
  placeholder,
  onChange,
  onSave,
}: {
  value: string;
  placeholder: string;
  onChange: (value: string) => void;
  onSave: () => void;
}) {
  return (
    <>
      <input
        type="password"
        className="w-full rounded-md border border-slate-700 bg-slate-900 px-2 py-1 text-xs outline-none focus:border-indigo-500"
        placeholder={placeholder}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") onSave();
        }}
      />
      <button
        type="button"
        className="w-full rounded-md border border-indigo-700 bg-indigo-950/40 px-2 py-0.5 text-[11px] hover:border-indigo-500"
        onClick={onSave}
      >
        保存
      </button>
    </>
  );
}

export function WebSearchSection({ enabled, onStatusChange }: WebSearchSectionProps) {
  const [apiKeyInput, setApiKeyInput] = useState("");
  const [keyError, setKeyError] = useState<string>();
  const [showReplace, setShowReplace] = useState(false);
  const detailsRef = useRef<HTMLDetailsElement>(null);
  const [expanded, setExpanded] = useState(!enabled);

  useEffect(() => {
    setApiKeyInput("");
    setKeyError(undefined);
    setShowReplace(false);
  }, [enabled]);

  useEffect(() => {
    setExpanded(!enabled);
  }, [enabled]);

  async function saveApiKey() {
    const value = apiKeyInput.trim();
    if (!value) {
      setKeyError("请输入 Tavily API Key");
      return;
    }
    try {
      await invoke("set_api_key", { provider: TAVILY_PROVIDER, apiKey: value });
      onStatusChange(true);
      setApiKeyInput("");
      setKeyError(undefined);
      setShowReplace(false);
    } catch (error) {
      setKeyError(String(error));
    }
  }

  async function clearApiKey() {
    try {
      await invoke("clear_api_key", { provider: TAVILY_PROVIDER });
      onStatusChange(false);
      setApiKeyInput("");
      setKeyError(undefined);
      setShowReplace(false);
    } catch (error) {
      setKeyError(String(error));
    }
  }

  return (
    <details
      id="sidebar-web-search"
      ref={detailsRef}
      open={expanded}
      onToggle={(e) => setExpanded(e.currentTarget.open)}
      className="shrink-0 rounded-md border border-slate-800 bg-slate-950/30"
    >
      <summary className="flex cursor-pointer list-none items-center justify-between px-2.5 py-2 text-[11px] marker:content-none [&::-webkit-details-marker]:hidden">
        <span className="uppercase tracking-[0.16em] text-slate-400">Web 搜索 (Tavily)</span>
        <span className={enabled ? "text-emerald-400" : "text-slate-500"}>
          {enabled ? "已启用" : "未启用"}
        </span>
      </summary>
      <div className="space-y-1.5 border-t border-slate-800 px-2 pb-2 pt-1.5">
        {enabled ? (
          <div className="rounded-md border border-slate-800 bg-slate-950/40 p-1.5">
            <div className="flex items-center justify-between text-[11px]">
              <span className="text-slate-400">Tavily API Key</span>
              <span className="text-emerald-400">已保存</span>
            </div>
            <div className="mt-1 flex items-center justify-end gap-1.5">
              <button
                type="button"
                className="rounded border border-slate-700 px-1.5 py-0.5 text-[11px] text-slate-400 hover:border-slate-500 hover:text-slate-200"
                onClick={() => setShowReplace((v) => !v)}
              >
                更换
              </button>
              <button
                type="button"
                className="rounded border border-slate-700 px-1.5 py-0.5 text-[11px] text-slate-400 hover:border-rose-500 hover:text-rose-300"
                onClick={() => void clearApiKey()}
              >
                清空
              </button>
            </div>
            {showReplace && (
              <div className="mt-1 space-y-1">
                <KeyEntry
                  value={apiKeyInput}
                  placeholder="输入新 Key 可覆盖保存"
                  onChange={(v) => {
                    setApiKeyInput(v);
                    setKeyError(undefined);
                  }}
                  onSave={() => void saveApiKey()}
                />
              </div>
            )}
            {keyError && <div className="mt-1 text-[11px] text-rose-400">{keyError}</div>}
          </div>
        ) : (
          <div className="rounded-md border border-slate-800 bg-slate-950/40 p-1.5">
            <div className="flex items-center justify-between text-[11px]">
              <span className="text-slate-400">Tavily API Key</span>
              <span className="text-amber-400">未配置</span>
            </div>
            <div className="mt-1 space-y-1">
              <KeyEntry
                value={apiKeyInput}
                placeholder="输入 Tavily API Key"
                onChange={(v) => {
                  setApiKeyInput(v);
                  setKeyError(undefined);
                }}
                onSave={() => void saveApiKey()}
              />
              {keyError && <div className="text-[11px] text-rose-400">{keyError}</div>}
            </div>
          </div>
        )}
      </div>
    </details>
  );
}
