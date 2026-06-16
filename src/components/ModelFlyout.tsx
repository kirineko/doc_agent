import { useEffect, useMemo, useRef, useState } from "react";
import type { SessionConfig } from "../lib/sessionConfig";
import { configForProviderFirstModel } from "../lib/sessionConfig";
import { useAnchorPosition } from "../hooks/useAnchorPosition";
import type { ModelInfo } from "../types";
import { providerLabel } from "../types";

interface ModelFlyoutProps {
  open: boolean;
  triggerRef: React.RefObject<HTMLButtonElement | null>;
  models: ModelInfo[];
  config: SessionConfig;
  locked: boolean;
  apiKeyStatus: Record<string, boolean>;
  onClose: () => void;
  onChange: (patch: Partial<SessionConfig>) => void;
}

function formatModelLabel(models: ModelInfo[], config: SessionConfig): string {
  const model = models.find((item) => item.id === config.model);
  const name = model?.label ?? config.model;
  if (!config.thinking_enabled) return `${name} · 思考关闭`;
  if (model?.supports_effort) return `${name} · ${config.thinking_effort}`;
  return name;
}

export function ModelFlyout({
  open,
  triggerRef,
  models,
  config,
  locked,
  apiKeyStatus,
  onClose,
  onChange,
}: ModelFlyoutProps) {
  const panelRef = useRef<HTMLDivElement>(null);
  const providers = useMemo(() => [...new Set(models.map((model) => model.provider))], [models]);
  const activeModel = models.find((model) => model.id === config.model);
  const [selectedProvider, setSelectedProvider] = useState(activeModel?.provider ?? providers[0] ?? "deepseek");
  const position = useAnchorPosition(triggerRef, open);
  const providerModels = models.filter((model) => model.provider === selectedProvider);
  const modelSelectedHere = activeModel?.provider === selectedProvider;

  useEffect(() => {
    if (!open) return;
    setSelectedProvider(activeModel?.provider ?? providers[0] ?? "deepseek");
  }, [open, activeModel?.provider, providers]);

  useEffect(() => {
    if (!open) return;

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") onClose();
    }

    function handlePointerDown(event: MouseEvent) {
      const target = event.target as Node;
      if (panelRef.current?.contains(target)) return;
      if (triggerRef.current?.contains(target)) return;
      onClose();
    }

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("mousedown", handlePointerDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("mousedown", handlePointerDown);
    };
  }, [open, onClose, triggerRef]);

  function handleProviderSelect(provider: string) {
    setSelectedProvider(provider);
    if (locked) return;
    if (activeModel?.provider === provider) return;
    const patch = configForProviderFirstModel(models, provider);
    if (patch) onChange(patch);
  }

  if (!open || !position) return null;

  return (
    <>
      <div className="fixed inset-0 z-40 bg-black/20" aria-hidden onClick={onClose} />
      <div
        ref={panelRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby="model-flyout-title"
        className="panel fixed z-50 flex flex-col overflow-hidden rounded-lg border border-border shadow-xl"
        style={{
          left: position.left,
          width: position.width,
          maxHeight: position.maxHeight,
          ...(position.top !== undefined ? { top: position.top } : {}),
          ...(position.bottom !== undefined ? { bottom: position.bottom } : {}),
        }}
      >
        <div className="shrink-0 border-b border-border-subtle px-3 py-2.5">
          <div className="flex items-start justify-between gap-2">
            <div className="min-w-0">
              <h2 id="model-flyout-title" className="text-xs font-semibold text-fg">
                模型
              </h2>
              <p className="mt-0.5 truncate text-[11px] text-fg-secondary">
                {formatModelLabel(models, config)}
              </p>
              {locked && (
                <p className="mt-0.5 text-[10px] text-fg-muted">对话开始后不可更改</p>
              )}
            </div>
            <button
              type="button"
              className="shrink-0 rounded border border-border-subtle px-1.5 py-0.5 text-[10px] text-fg-secondary hover:border-border-hover"
              onClick={onClose}
            >
              关闭
            </button>
          </div>
        </div>

        {locked ? (
          <div className="px-3 py-2 text-[11px] text-fg-secondary">
            当前会话已锁定模型配置。
          </div>
        ) : (
          <>
            <div className="shrink-0 px-3 pt-2">
              <div className="text-[10px] uppercase tracking-[0.14em] text-fg-muted">Provider</div>
              <div className="mt-1 flex gap-1 rounded-md border border-border-subtle p-0.5">
                {providers.map((provider) => (
                  <button
                    key={provider}
                    type="button"
                    className={`relative flex-1 rounded px-2 py-1.5 text-[11px] transition-colors ${
                      selectedProvider === provider
                        ? "bg-accent/15 font-medium text-fg"
                        : "text-fg-secondary hover:bg-surface-hover hover:text-fg"
                    }`}
                    onClick={() => handleProviderSelect(provider)}
                  >
                    {providerLabel(provider)}
                    {!apiKeyStatus[provider] && (
                      <span
                        aria-hidden
                        className="absolute right-1 top-1 h-1.5 w-1.5 rounded-full bg-amber-500"
                        title="未配置 API Key"
                      />
                    )}
                  </button>
                ))}
              </div>
            </div>

            <div className="min-h-0 flex-1 overflow-y-auto px-3 py-2">
              <div className="text-[10px] uppercase tracking-[0.14em] text-fg-muted">选择模型</div>
              <div className="mt-1 space-y-1">
                {providerModels.map((model) => {
                  const selected = config.model === model.id;
                  return (
                    <label
                      key={model.id}
                      className={`flex cursor-pointer items-center gap-2 rounded-md border px-2.5 py-2 text-xs transition-colors ${
                        selected
                          ? "border-accent/60 border-l-2 border-l-accent bg-accent/10 text-fg"
                          : "border-border-subtle text-fg-secondary hover:border-border-hover"
                      }`}
                    >
                      <input
                        type="radio"
                        className="sr-only"
                        name={`model-${selectedProvider}`}
                        checked={selected}
                        onChange={() => onChange({ model: model.id })}
                      />
                      <span className="flex-1">{model.label}</span>
                      {model.supports_vision && (
                        <span className="text-[10px] text-sky-500" title="支持视觉">
                          视觉
                        </span>
                      )}
                    </label>
                  );
                })}
              </div>
            </div>

            <div className="shrink-0 space-y-2 border-t border-border-subtle bg-panel px-3 py-2.5">
              <div className="text-[10px] uppercase tracking-[0.14em] text-fg-muted">思考</div>
              {modelSelectedHere ? (
                <>
                  <label className="flex items-center gap-2 text-xs text-fg">
                    <input
                      type="checkbox"
                      checked={config.thinking_enabled}
                      onChange={(event) => onChange({ thinking_enabled: event.target.checked })}
                    />
                    启用思考模式
                  </label>
                  {activeModel?.supports_effort && config.thinking_enabled && (
                    <select
                      className="input-field w-full rounded-md px-2.5 py-1.5 text-xs"
                      value={config.thinking_effort}
                      onChange={(event) => onChange({ thinking_effort: event.target.value })}
                    >
                      <option value="high">high</option>
                      <option value="max">max</option>
                    </select>
                  )}
                  {!activeModel?.supports_effort && config.thinking_enabled && (
                    <p className="text-[10px] text-fg-muted">当前模型不支持调节思考强度</p>
                  )}
                </>
              ) : (
                <p className="text-[10px] text-fg-muted">选择此 Provider 的模型后可配置思考选项</p>
              )}
            </div>
          </>
        )}
      </div>
    </>
  );
}
