import { canonicalModelId, findModel, formatModelThinkingLabel, providerLabel, selectableModels, type ModelInfo } from "../types";
import { MODEL_OPTIONS } from "../types";
import type { SessionConfig } from "../lib/sessionConfig";
import { defaultsForModel } from "../lib/sessionConfig";
import { ChoiceChipGroup } from "./ChoiceChipGroup";

interface ModelConfigSectionProps {
  config: SessionConfig;
  locked: boolean;
  models?: ModelInfo[];
  onChange: (patch: Partial<SessionConfig>) => void;
}

export function ModelConfigSection({
  config,
  locked,
  models = MODEL_OPTIONS,
  onChange,
}: ModelConfigSectionProps) {
  const activeModel = findModel(models, config.model);
  const options = selectableModels(models);

  if (locked) {
    return (
      <div className="shrink-0 space-y-1 border-t border-border pt-2.5">
        <div className="text-[11px] uppercase tracking-[0.16em] text-fg-secondary">模型</div>
        <div className="config-surface rounded-md px-2.5 py-2 text-xs text-fg">
          {formatModelThinkingLabel(activeModel, config)}
        </div>
        {activeModel?.availability === "retired" && (
          <div className="text-[10px] text-amber-600">
            该模型已不可调用。请新建会话选择 MiMo v2.5 Pro。
          </div>
        )}
      </div>
    );
  }

  return (
    <div className="shrink-0 space-y-2 border-t border-border pt-2.5">
      <div className="text-[11px] uppercase tracking-[0.16em] text-fg-secondary">模型</div>
      <select
        className="input-field w-full rounded-md px-2.5 py-1.5 text-xs"
        value={canonicalModelId(config.model)}
        onChange={(e) => {
          const next = models.find((item) => item.id === e.target.value);
          onChange(next ? defaultsForModel(next) : { model: e.target.value });
        }}
      >
        {options.map((model) => (
          <option key={model.id} value={model.id}>
            {model.label}
          </option>
        ))}
      </select>

      {activeModel?.supports_thinking_toggle ? (
        <label className="flex items-center gap-2 text-xs">
          <input
            type="checkbox"
            checked={config.thinking_enabled}
            onChange={(e) => onChange({ thinking_enabled: e.target.checked })}
          />
          启用思考模式
        </label>
      ) : (
        <div className="text-[10px] text-fg-muted">此模型始终启用思考</div>
      )}

      {Boolean(activeModel?.thinking_efforts.length) &&
        (config.thinking_enabled || !activeModel?.supports_thinking_toggle) && (
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

      <div className="text-[10px] text-fg-muted">
        对话开始后模型不可切换 · {providerLabel(activeModel?.provider ?? "")}
      </div>
    </div>
  );
}
