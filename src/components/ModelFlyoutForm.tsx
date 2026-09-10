import type { SessionConfig } from "../lib/sessionConfig";
import { defaultsForModel } from "../lib/sessionConfig";
import type { ModelInfo } from "../types";
import { canonicalModelId, providerLabel } from "../types";
import { ChoiceChipGroup } from "./ChoiceChipGroup";

interface ModelFlyoutFormProps {
  providers: string[];
  selectedProvider: string;
  providerModels: ModelInfo[];
  apiKeyStatus: Record<string, boolean>;
  config: SessionConfig;
  activeModel: ModelInfo | undefined;
  modelSelectedHere: boolean;
  onProviderSelect: (provider: string) => void;
  onChange: (patch: Partial<SessionConfig>) => void;
}

export function ModelFlyoutForm({
  providers,
  selectedProvider,
  providerModels,
  apiKeyStatus,
  config,
  activeModel,
  modelSelectedHere,
  onProviderSelect,
  onChange,
}: ModelFlyoutFormProps) {
  const showEffort =
    Boolean(activeModel?.thinking_efforts?.length) &&
    (config.thinking_enabled || !activeModel?.supports_thinking_toggle);

  return (
    <>
      <div className="shrink-0 px-3 pt-2">
        <div className="text-[10px] uppercase tracking-[0.14em] text-fg-muted">Provider</div>
        <div className="mt-1">
          <ChoiceChipGroup
            label="Provider"
            layout="grid"
            value={selectedProvider}
            options={providers.map((provider) => ({
              value: provider,
              label: providerLabel(provider),
              hint: apiKeyStatus[provider] ? undefined : "未配置 API Key",
            }))}
            onChange={onProviderSelect}
          />
        </div>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto px-3 py-2">
        <div className="text-[10px] uppercase tracking-[0.14em] text-fg-muted">选择模型</div>
        <div className="mt-1 space-y-1">
          {providerModels.map((model) => {
            const selected = canonicalModelId(config.model) === model.id;
            return (
              <label
                key={model.id}
                className={`flex cursor-pointer items-center gap-2 rounded-md border px-2.5 py-2 text-xs transition-colors ${
                  selected
                    ? "border-accent bg-accent-muted text-fg"
                    : "border-border-subtle text-fg-secondary hover:border-border-hover"
                }`}
              >
                <input
                  type="radio"
                  className="sr-only"
                  name={`model-${selectedProvider}`}
                  checked={selected}
                  onChange={() => onChange(defaultsForModel(model))}
                />
                <span className="flex-1">{model.label}</span>
                {model.supports_vision && (
                  <span className="text-[10px] text-link" title="支持视觉">
                    视觉
                  </span>
                )}
              </label>
            );
          })}
        </div>
      </div>

      <div className="shrink-0 space-y-2 border-t border-border-subtle bg-muted px-3 py-2.5">
        <div className="text-[10px] uppercase tracking-[0.14em] text-fg-muted">思考</div>
        {modelSelectedHere ? (
          <>
            {activeModel?.supports_thinking_toggle ? (
              <label className="flex items-center gap-2 text-xs text-fg">
                <input
                  type="checkbox"
                  checked={config.thinking_enabled}
                  onChange={(event) => onChange({ thinking_enabled: event.target.checked })}
                />
                启用思考模式
              </label>
            ) : (
              <p className="text-[10px] text-fg-muted">此模型始终启用思考</p>
            )}
            {showEffort && (
              <ChoiceChipGroup
                label="思考强度"
                layout="row"
                value={config.thinking_effort}
                options={(activeModel?.thinking_efforts ?? []).map((effort) => ({
                  value: effort,
                  label: effort,
                }))}
                onChange={(thinking_effort) => onChange({ thinking_effort })}
              />
            )}
            {!activeModel?.thinking_efforts?.length && activeModel?.supports_thinking_toggle && (
              <p className="text-[10px] text-fg-muted">当前模型不支持调节思考强度</p>
            )}
          </>
        ) : (
          <p className="text-[10px] text-fg-muted">选择此 Provider 的模型后可配置思考选项</p>
        )}
      </div>
    </>
  );
}
