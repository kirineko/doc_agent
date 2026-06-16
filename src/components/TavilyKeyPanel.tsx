import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { KeyEntry } from "./KeyEntry";

const TAVILY_PROVIDER = "tavily";

interface TavilyKeyPanelProps {
  hasKey: boolean;
  highlighted?: boolean;
  onStatusChange: (has: boolean) => void;
}

export function TavilyKeyPanel({ hasKey, highlighted, onStatusChange }: TavilyKeyPanelProps) {
  const [apiKeyInput, setApiKeyInput] = useState("");
  const [keyError, setKeyError] = useState<string>();
  const [showReplace, setShowReplace] = useState(false);

  useEffect(() => {
    setApiKeyInput("");
    setKeyError(undefined);
    setShowReplace(false);
  }, [hasKey]);

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

  if (hasKey) {
    return (
      <div
        id="api-key-tavily"
        className={`config-surface rounded-md p-1.5 ${highlighted ? "border border-amber-600/80" : ""}`}
      >
        <div className="flex items-center justify-between text-[11px]">
          <span className="text-fg-secondary">Tavily</span>
          <span className="text-emerald-600">已保存</span>
        </div>
        <div className="mt-1 flex items-center justify-end gap-1.5">
          <button
            type="button"
            className="rounded border border-border-subtle px-1.5 py-0.5 text-[11px] text-fg-secondary hover:border-border-hover hover:text-fg"
            onClick={() => setShowReplace((value) => !value)}
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
              onChange={(value) => {
                setApiKeyInput(value);
                setKeyError(undefined);
              }}
              onSave={() => void saveApiKey()}
            />
          </div>
        )}
        {keyError && <div className="mt-1 text-[11px] text-rose-500">{keyError}</div>}
      </div>
    );
  }

  return (
    <div
      id="api-key-tavily"
      className={`config-surface rounded-md p-1.5 ${highlighted ? "border border-amber-600/80 ring-1 ring-amber-600/40" : ""}`}
    >
      <div className="flex items-center justify-between text-[11px]">
        <span className="text-fg-secondary">Tavily</span>
        <span className="text-amber-600">未配置</span>
      </div>
      <div className="mt-1 space-y-1">
        <KeyEntry
          value={apiKeyInput}
          placeholder="输入 Tavily API Key"
          onChange={(value) => {
            setApiKeyInput(value);
            setKeyError(undefined);
          }}
          onSave={() => void saveApiKey()}
        />
        {keyError && <div className="text-[11px] text-rose-500">{keyError}</div>}
      </div>
    </div>
  );
}
