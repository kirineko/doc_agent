import { MODEL_OPTIONS, providerLabel } from "../types";
import type { SessionConfig } from "../lib/sessionConfig";

interface ModelConfigSectionProps {
  config: SessionConfig;
  locked: boolean;
  onChange: (patch: Partial<SessionConfig>) => void;
}

function formatModelLabel(config: SessionConfig): string {
  const model = MODEL_OPTIONS.find((m) => m.id === config.model);
  const name = model?.label ?? config.model;
  if (!config.thinking_enabled) {
    return `${name} · 思考关闭`;
  }
  if (model?.supportsEffort) {
    return `${name} · ${config.thinking_effort}`;
  }
  return name;
}

export function ModelConfigSection({ config, locked, onChange }: ModelConfigSectionProps) {
  const activeModel = MODEL_OPTIONS.find((m) => m.id === config.model);

  if (locked) {
    return (
      <div className="shrink-0 space-y-1 border-t border-slate-800 pt-2.5">
        <div className="text-[11px] uppercase tracking-[0.16em] text-slate-400">模型</div>
        <div className="rounded-md border border-slate-800 bg-slate-950/40 px-2.5 py-2 text-xs text-slate-300">
          {formatModelLabel(config)}
        </div>
      </div>
    );
  }

  return (
    <div className="shrink-0 space-y-2 border-t border-slate-800 pt-2.5">
      <div className="text-[11px] uppercase tracking-[0.16em] text-slate-400">模型</div>
      <select
        className="w-full rounded-md border border-slate-700 bg-slate-900 px-2.5 py-1.5 text-xs"
        value={config.model}
        onChange={(e) => onChange({ model: e.target.value })}
      >
        {MODEL_OPTIONS.map((model) => (
          <option key={model.id} value={model.id}>
            {model.label}
          </option>
        ))}
      </select>

      <label className="flex items-center gap-2 text-xs">
        <input
          type="checkbox"
          checked={config.thinking_enabled}
          onChange={(e) => onChange({ thinking_enabled: e.target.checked })}
        />
        启用思考模式
      </label>

      {activeModel?.supportsEffort && config.thinking_enabled && (
        <select
          className="w-full rounded-md border border-slate-700 bg-slate-900 px-2.5 py-1.5 text-xs"
          value={config.thinking_effort}
          onChange={(e) => onChange({ thinking_effort: e.target.value })}
        >
          <option value="high">high</option>
          <option value="max">max</option>
        </select>
      )}

      <div className="text-[10px] text-slate-500">
        对话开始后模型不可切换 · {providerLabel(activeModel?.provider ?? "")}
      </div>
    </div>
  );
}
