import type { SessionConfig } from "../lib/sessionConfig";
import type { ModelInfo } from "../types";
import { ProviderApiKeyPanel } from "./ProviderApiKeyPanel";

interface ProviderModelPanelProps {
  provider: string;
  models: ModelInfo[];
  config: SessionConfig;
  apiKeyStatus: Record<string, boolean>;
  highlightApiKey: boolean;
  onChange: (patch: Partial<SessionConfig>) => void;
  onApiKeyStatusChange: (provider: string, has: boolean) => void;
}

export function ProviderModelPanel({
  provider,
  models,
  config,
  apiKeyStatus,
  highlightApiKey,
  onChange,
  onApiKeyStatusChange,
}: ProviderModelPanelProps) {
  const providerModels = models.filter((model) => model.provider === provider);
  const activeModel = models.find((model) => model.id === config.model);
  const modelSelectedHere = activeModel?.provider === provider;

  return (
    <div className="space-y-3">
      <ProviderApiKeyPanel
        provider={provider}
        hasKey={Boolean(apiKeyStatus[provider])}
        highlighted={highlightApiKey}
        onStatusChange={onApiKeyStatusChange}
      />

      <div className="space-y-1.5">
        <div className="text-[10px] uppercase tracking-[0.14em] text-fg-muted">模型</div>
        <div className="space-y-1">
          {providerModels.map((model) => (
            <label
              key={model.id}
              className={`flex cursor-pointer items-center gap-2 rounded-md border px-2.5 py-1.5 text-xs ${
                config.model === model.id
                  ? "border-accent/60 bg-accent/10 text-fg"
                  : "border-border-subtle text-fg-secondary hover:border-border-hover"
              }`}
            >
              <input
                type="radio"
                className="sr-only"
                name={`model-${provider}`}
                checked={config.model === model.id}
                onChange={() => onChange({ model: model.id })}
              />
              <span className="flex-1">{model.label}</span>
              {model.supports_vision && (
                <span className="text-[10px] text-sky-500" title="支持视觉">
                  视觉
                </span>
              )}
            </label>
          ))}
        </div>
      </div>

      {modelSelectedHere && (
        <div className="space-y-2 border-t border-border-subtle pt-2">
          <div className="text-[10px] uppercase tracking-[0.14em] text-fg-muted">思考</div>
          <label className="flex items-center gap-2 text-xs text-fg">
            <input
              type="checkbox"
              checked={config.thinking_enabled}
              onChange={(event) => onChange({ thinking_enabled: event.target.checked })}
            />
            启用思考模式
          </label>
          {activeModel?.supports_effort && config.thinking_enabled && (
            <div className="space-y-1">
              <div className="text-[10px] text-fg-muted">思考强度</div>
              <select
                className="input-field w-full rounded-md px-2.5 py-1.5 text-xs"
                value={config.thinking_effort}
                onChange={(event) => onChange({ thinking_effort: event.target.value })}
              >
                <option value="high">high</option>
                <option value="max">max</option>
              </select>
            </div>
          )}
          {!activeModel?.supports_effort && config.thinking_enabled && (
            <p className="text-[10px] text-fg-muted">当前模型不支持调节思考强度</p>
          )}
        </div>
      )}

      {!modelSelectedHere && (
        <p className="text-[10px] text-fg-muted">选择此 Provider 的模型后可配置思考选项</p>
      )}
    </div>
  );
}
