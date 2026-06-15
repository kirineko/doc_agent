import { useEffect, useMemo, useState } from "react";
import type { SessionConfig } from "../lib/sessionConfig";
import { configForProviderFirstModel } from "../lib/sessionConfig";
import type { ModelInfo } from "../types";
import { providerLabel } from "../types";
import { ProviderApiKeyPanel } from "./ProviderApiKeyPanel";
import { ProviderModelPanel } from "./ProviderModelPanel";

interface ModelSettingsDrawerProps {
  open: boolean;
  models: ModelInfo[];
  config: SessionConfig;
  locked: boolean;
  apiKeyStatus: Record<string, boolean>;
  highlightApiKeyProvider?: string;
  onClose: () => void;
  onChange: (patch: Partial<SessionConfig>) => void;
  onApiKeyStatusChange: (provider: string, has: boolean) => void;
}

function formatModelLabel(models: ModelInfo[], config: SessionConfig): string {
  const model = models.find((item) => item.id === config.model);
  const name = model?.label ?? config.model;
  if (!config.thinking_enabled) return `${name} · 思考关闭`;
  if (model?.supports_effort) return `${name} · ${config.thinking_effort}`;
  return name;
}

export function ModelSettingsDrawer({
  open,
  models,
  config,
  locked,
  apiKeyStatus,
  highlightApiKeyProvider,
  onClose,
  onChange,
  onApiKeyStatusChange,
}: ModelSettingsDrawerProps) {
  const providers = useMemo(() => [...new Set(models.map((model) => model.provider))], [models]);
  const activeModel = models.find((model) => model.id === config.model);
  const [selectedProvider, setSelectedProvider] = useState(activeModel?.provider ?? providers[0] ?? "deepseek");

  useEffect(() => {
    if (!open) return;
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") onClose();
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [open, onClose]);

  useEffect(() => {
    if (!open) return;
    if (highlightApiKeyProvider) {
      setSelectedProvider(highlightApiKeyProvider);
      return;
    }
    setSelectedProvider(activeModel?.provider ?? providers[0] ?? "deepseek");
  }, [open, activeModel?.provider, highlightApiKeyProvider, providers]);

  function handleProviderSelect(provider: string) {
    setSelectedProvider(provider);
    if (locked) return;
    if (activeModel?.provider === provider) return;
    const patch = configForProviderFirstModel(models, provider);
    if (patch) onChange(patch);
  }

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex justify-end">
      <button
        type="button"
        aria-label="关闭模型与密钥"
        className="absolute inset-0 bg-black/35"
        onClick={onClose}
      />
      <aside
        role="dialog"
        aria-modal="true"
        aria-labelledby="model-settings-title"
        className="panel relative flex h-full w-80 shrink-0 flex-col gap-3 overflow-y-auto border-l border-border p-4 shadow-xl"
      >
        <div className="flex items-center justify-between">
          <h2 id="model-settings-title" className="text-sm font-semibold text-fg">
            模型与密钥
          </h2>
          <button
            type="button"
            className="rounded-md border border-border-subtle px-2 py-1 text-xs text-fg-secondary hover:border-border-hover hover:text-fg"
            onClick={onClose}
          >
            关闭
          </button>
        </div>

        {locked ? (
          <>
            <div className="config-surface rounded-md px-2.5 py-2 text-xs text-fg">
              {formatModelLabel(models, config)}
            </div>
            <div className="flex gap-1 rounded-md border border-border-subtle p-1">
              {providers.map((provider) => (
                <button
                  key={provider}
                  type="button"
                  className={`flex-1 rounded px-2 py-1.5 text-xs transition-colors ${
                    selectedProvider === provider
                      ? "bg-accent/15 font-medium text-fg"
                      : "text-fg-secondary hover:bg-surface-hover hover:text-fg"
                  }`}
                  onClick={() => handleProviderSelect(provider)}
                >
                  {providerLabel(provider)}
                </button>
              ))}
            </div>
            <ProviderApiKeyPanel
              provider={selectedProvider}
              hasKey={Boolean(apiKeyStatus[selectedProvider])}
              highlighted={highlightApiKeyProvider === selectedProvider}
              onStatusChange={onApiKeyStatusChange}
            />
          </>
        ) : (
          <>
            <div className="flex gap-1 rounded-md border border-border-subtle p-1">
              {providers.map((provider) => (
                <button
                  key={provider}
                  type="button"
                  className={`flex-1 rounded px-2 py-1.5 text-xs transition-colors ${
                    selectedProvider === provider
                      ? "bg-accent/15 font-medium text-fg"
                      : "text-fg-secondary hover:bg-surface-hover hover:text-fg"
                  }`}
                  onClick={() => handleProviderSelect(provider)}
                >
                  {providerLabel(provider)}
                </button>
              ))}
            </div>

            <ProviderModelPanel
              provider={selectedProvider}
              models={models}
              config={config}
              apiKeyStatus={apiKeyStatus}
              highlightApiKey={highlightApiKeyProvider === selectedProvider}
              onChange={onChange}
              onApiKeyStatusChange={onApiKeyStatusChange}
            />

            <p className="text-[10px] text-fg-muted">对话开始后模型不可切换</p>
          </>
        )}
      </aside>
    </div>
  );
}
