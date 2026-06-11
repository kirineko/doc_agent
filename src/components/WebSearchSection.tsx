import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { KeyEntry } from "./KeyEntry";

const TAVILY_PROVIDER = "tavily";

interface WebSearchSectionProps {
  enabled: boolean;
  onStatusChange: (has: boolean) => void;
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
      className="config-surface shrink-0 rounded-md"
    >
      <summary className="flex cursor-pointer list-none items-center justify-between px-2.5 py-2 text-[11px] marker:content-none [&::-webkit-details-marker]:hidden">
        <span className="uppercase tracking-[0.16em] text-fg-secondary">Web 搜索 (Tavily)</span>
        <span className={enabled ? "text-emerald-600" : "text-fg-muted"}>
          {enabled ? "已启用" : "未启用"}
        </span>
      </summary>
      <div className="space-y-1.5 border-t border-border px-2 pb-2 pt-1.5">
        {enabled ? (
          <div className="config-surface rounded-md p-1.5">
            <div className="flex items-center justify-between text-[11px]">
              <span className="text-fg-secondary">Tavily API Key</span>
              <span className="text-emerald-600">已保存</span>
            </div>
            <div className="mt-1 flex items-center justify-end gap-1.5">
              <button
                type="button"
                className="rounded border border-border-subtle px-1.5 py-0.5 text-[11px] text-fg-secondary hover:border-border-hover hover:text-fg"
                onClick={() => setShowReplace((v) => !v)}
              >
                更换
              </button>
              <button
                type="button"
                className="rounded border border-border-subtle px-1.5 py-0.5 text-[11px] text-fg-secondary hover:border-rose-500 hover:text-rose-500"
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
            {keyError && <div className="mt-1 text-[11px] text-rose-500">{keyError}</div>}
          </div>
        ) : (
          <div className="config-surface rounded-md p-1.5">
            <div className="flex items-center justify-between text-[11px]">
              <span className="text-fg-secondary">Tavily API Key</span>
              <span className="text-amber-600">未配置</span>
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
              {keyError && <div className="text-[11px] text-rose-500">{keyError}</div>}
            </div>
          </div>
        )}
      </div>
    </details>
  );
}
