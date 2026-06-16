import { useEffect } from "react";
import { ApiKeySection } from "./ApiKeySection";
import { TavilyKeyPanel } from "./TavilyKeyPanel";

interface CredentialsDrawerProps {
  open: boolean;
  apiKeyStatus: Record<string, boolean>;
  tavilyHasKey: boolean;
  highlightApiKeyProvider?: string;
  onClose: () => void;
  onApiKeyStatusChange: (provider: string, has: boolean) => void;
  onTavilyStatusChange: (has: boolean) => void;
}

export function CredentialsDrawer({
  open,
  apiKeyStatus,
  tavilyHasKey,
  highlightApiKeyProvider,
  onClose,
  onApiKeyStatusChange,
  onTavilyStatusChange,
}: CredentialsDrawerProps) {
  useEffect(() => {
    if (!open) return;

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") onClose();
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex justify-end">
      <button
        type="button"
        aria-label="关闭密钥与服务"
        className="absolute inset-0 bg-black/35"
        onClick={onClose}
      />
      <aside
        role="dialog"
        aria-modal="true"
        aria-labelledby="credentials-drawer-title"
        className="panel relative flex h-full w-80 shrink-0 flex-col gap-3 overflow-y-auto border-l border-border p-4 shadow-xl"
      >
        <div className="flex items-center justify-between">
          <h2 id="credentials-drawer-title" className="text-sm font-semibold text-fg">
            密钥与服务
          </h2>
          <button
            type="button"
            className="rounded-md border border-border-subtle px-2 py-1 text-xs text-fg-secondary hover:border-border-hover hover:text-fg"
            onClick={onClose}
          >
            关闭
          </button>
        </div>

        <section className="space-y-2">
          <div className="text-[11px] uppercase tracking-[0.16em] text-fg-secondary">LLM</div>
          <ApiKeySection
            apiKeyStatus={apiKeyStatus}
            highlightProvider={highlightApiKeyProvider}
            onApiKeyStatusChange={onApiKeyStatusChange}
          />
        </section>

        <section className="space-y-2">
          <div className="text-[11px] uppercase tracking-[0.16em] text-fg-secondary">搜索服务</div>
          <TavilyKeyPanel
            hasKey={tavilyHasKey}
            highlighted={highlightApiKeyProvider === "tavily"}
            onStatusChange={onTavilyStatusChange}
          />
        </section>

        <p className="text-[10px] text-fg-muted">API Key 保存在系统钥匙串，不会写入项目或日志。</p>
      </aside>
    </div>
  );
}
