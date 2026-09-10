import { useEffect, useMemo, useRef, useState } from "react";
import type { SessionConfig } from "../lib/sessionConfig";
import { configForProviderFirstModel } from "../lib/sessionConfig";
import { useAnchorPosition } from "../hooks/useAnchorPosition";
import type { ModelInfo } from "../types";
import { findModel, formatModelThinkingLabel, selectableModels } from "../types";
import { ModelFlyoutForm } from "./ModelFlyoutForm";

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
  const choosable = useMemo(() => selectableModels(models), [models]);
  const providers = useMemo(() => [...new Set(choosable.map((model) => model.provider))], [choosable]);
  const activeModel = findModel(models, config.model);
  const [selectedProvider, setSelectedProvider] = useState(activeModel?.provider ?? providers[0] ?? "deepseek");
  const position = useAnchorPosition(triggerRef, open, { minWidth: 300, maxHeight: 480 });
  const providerModels = choosable.filter((model) => model.provider === selectedProvider);
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
                {formatModelThinkingLabel(activeModel, config)}
              </p>
              {locked && <p className="mt-0.5 text-[10px] text-fg-muted">对话开始后不可更改</p>}
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
          <div className="px-3 py-2 text-[11px] text-fg-secondary">当前会话已锁定模型配置。</div>
        ) : (
          <ModelFlyoutForm
            providers={providers}
            selectedProvider={selectedProvider}
            providerModels={providerModels}
            apiKeyStatus={apiKeyStatus}
            config={config}
            activeModel={activeModel}
            modelSelectedHere={modelSelectedHere}
            onProviderSelect={handleProviderSelect}
            onChange={onChange}
          />
        )}
      </div>
    </>
  );
}
