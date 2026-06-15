import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { providerLabel } from "../types";
import { KeyEntry } from "./KeyEntry";

interface ProviderKeyRowProps {
  provider: string;
  hasKey: boolean;
  highlighted: boolean;
  onStatusChange: (provider: string, has: boolean) => void;
}

export function ProviderKeyRow({
  provider,
  hasKey,
  highlighted,
  onStatusChange,
}: ProviderKeyRowProps) {
  const [apiKeyInput, setApiKeyInput] = useState("");
  const [keyError, setKeyError] = useState<string>();
  const [showReplace, setShowReplace] = useState(false);

  useEffect(() => {
    setApiKeyInput("");
    setKeyError(undefined);
    setShowReplace(false);
  }, [provider, hasKey]);

  async function saveApiKey() {
    const value = apiKeyInput.trim();
    if (!value) {
      setKeyError("请输入 API Key");
      return;
    }
    try {
      await invoke("set_api_key", { provider, apiKey: value });
      onStatusChange(provider, true);
      setApiKeyInput("");
      setKeyError(undefined);
      setShowReplace(false);
    } catch (error) {
      setKeyError(String(error));
    }
  }

  async function clearApiKey() {
    try {
      await invoke("clear_api_key", { provider });
      onStatusChange(provider, false);
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
        id={`api-key-${provider}`}
        className={`rounded-md p-1.5 ${highlighted ? "border border-amber-600/80" : ""}`}
      >
        <div className="flex items-center justify-between text-[11px]">
          <span className="text-fg-secondary">{providerLabel(provider)}</span>
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
      id={`api-key-${provider}`}
      className={`rounded-md p-1.5 ${highlighted ? "border border-amber-600/80 ring-1 ring-amber-600/40" : ""}`}
    >
      <div className="flex items-center justify-between text-[11px]">
        <span className="text-fg-secondary">{providerLabel(provider)}</span>
        <span className="text-amber-600">未配置</span>
      </div>
      <div className="mt-1 space-y-1">
        <KeyEntry
          value={apiKeyInput}
          placeholder="输入 API Key"
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
