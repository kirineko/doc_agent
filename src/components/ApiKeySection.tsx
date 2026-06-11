import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { API_PROVIDERS, providerLabel } from "../types";

interface ApiKeySectionProps {
  apiKeyStatus: Record<string, boolean>;
  highlightProvider?: string;
  onApiKeyStatusChange: (provider: string, has: boolean) => void;
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

function ProviderKeyRow({
  provider,
  hasKey,
  highlighted,
  onStatusChange,
}: {
  provider: string;
  hasKey: boolean;
  highlighted: boolean;
  onStatusChange: (provider: string, has: boolean) => void;
}) {
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
        className={`rounded-md border bg-slate-950/40 p-1.5 ${highlighted ? "border-amber-600/80" : "border-slate-800"}`}
      >
        <div className="flex items-center justify-between text-[11px]">
          <span className="text-slate-400">{providerLabel(provider)}</span>
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
    );
  }

  return (
    <div
      id={`api-key-${provider}`}
      className={`rounded-md border bg-slate-950/40 p-1.5 ${highlighted ? "border-amber-600/80 ring-1 ring-amber-600/40" : "border-slate-800"}`}
    >
      <div className="flex items-center justify-between text-[11px]">
        <span className="text-slate-400">{providerLabel(provider)}</span>
        <span className="text-amber-400">未配置</span>
      </div>
      <div className="mt-1 space-y-1">
        <KeyEntry
          value={apiKeyInput}
          placeholder="输入 API Key"
          onChange={(v) => {
            setApiKeyInput(v);
            setKeyError(undefined);
          }}
          onSave={() => void saveApiKey()}
        />
        {keyError && <div className="text-[11px] text-rose-400">{keyError}</div>}
      </div>
    </div>
  );
}

function configuredSummary(apiKeyStatus: Record<string, boolean>): string {
  const configured = API_PROVIDERS.filter((p) => apiKeyStatus[p]).map((p) => providerLabel(p));
  if (configured.length === 0) return "未配置";
  return configured.join("、");
}

export function ApiKeySection({
  apiKeyStatus,
  highlightProvider,
  onApiKeyStatusChange,
}: ApiKeySectionProps) {
  const hasAnyKey = API_PROVIDERS.some((p) => apiKeyStatus[p]);
  const detailsRef = useRef<HTMLDetailsElement>(null);
  const [expanded, setExpanded] = useState(!hasAnyKey);

  useEffect(() => {
    setExpanded(!hasAnyKey);
  }, [hasAnyKey]);

  useEffect(() => {
    if (!highlightProvider) return;
    setExpanded(true);
    requestAnimationFrame(() => {
      detailsRef.current?.scrollIntoView({ block: "nearest" });
      document.getElementById(`api-key-${highlightProvider}`)?.scrollIntoView({ block: "nearest" });
    });
  }, [highlightProvider]);

  return (
    <details
      id="sidebar-api-keys"
      ref={detailsRef}
      open={expanded}
      onToggle={(e) => setExpanded(e.currentTarget.open)}
      className={`shrink-0 rounded-md border bg-slate-950/30 ${highlightProvider ? "border-amber-600/60" : "border-slate-800"}`}
    >
      <summary className="flex cursor-pointer list-none items-center justify-between px-2.5 py-2 text-[11px] marker:content-none [&::-webkit-details-marker]:hidden">
        <span className="uppercase tracking-[0.16em] text-slate-400">API Key</span>
        <span className={hasAnyKey ? "text-emerald-400" : "text-amber-400"}>
          {configuredSummary(apiKeyStatus)}
        </span>
      </summary>
      <div className="space-y-1.5 border-t border-slate-800 px-2 pb-2 pt-1.5">
        {API_PROVIDERS.map((provider) => (
          <ProviderKeyRow
            key={provider}
            provider={provider}
            hasKey={Boolean(apiKeyStatus[provider])}
            highlighted={highlightProvider === provider}
            onStatusChange={onApiKeyStatusChange}
          />
        ))}
      </div>
    </details>
  );
}
