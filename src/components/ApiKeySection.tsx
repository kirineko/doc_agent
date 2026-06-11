import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { API_PROVIDERS, providerLabel } from "../types";
import { KeyEntry } from "./KeyEntry";

interface ApiKeySectionProps {
  apiKeyStatus: Record<string, boolean>;
  highlightProvider?: string;
  onApiKeyStatusChange: (provider: string, has: boolean) => void;
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
        className={`config-surface rounded-md p-1.5 ${highlighted ? "border-amber-600/80" : ""}`}
      >
        <div className="flex items-center justify-between text-[11px]">
          <span className="text-fg-secondary">{providerLabel(provider)}</span>
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
    );
  }

  return (
    <div
      id={`api-key-${provider}`}
      className={`config-surface rounded-md p-1.5 ${highlighted ? "border-amber-600/80 ring-1 ring-amber-600/40" : ""}`}
    >
      <div className="flex items-center justify-between text-[11px]">
        <span className="text-fg-secondary">{providerLabel(provider)}</span>
        <span className="text-amber-600">未配置</span>
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
        {keyError && <div className="text-[11px] text-rose-500">{keyError}</div>}
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
      className={`config-surface shrink-0 rounded-md ${highlightProvider ? "border-amber-600/60" : ""}`}
    >
      <summary className="flex cursor-pointer list-none items-center justify-between px-2.5 py-2 text-[11px] marker:content-none [&::-webkit-details-marker]:hidden">
        <span className="uppercase tracking-[0.16em] text-fg-secondary">API Key</span>
        <span className={hasAnyKey ? "text-emerald-600" : "text-amber-600"}>
          {configuredSummary(apiKeyStatus)}
        </span>
      </summary>
      <div className="space-y-1.5 border-t border-border px-2 pb-2 pt-1.5">
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
