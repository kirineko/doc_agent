interface WebSearchStatusProps {
  enabled: boolean;
  onEnable: () => void;
  onDisable: () => void;
}

export function WebSearchStatus({ enabled, onEnable, onDisable }: WebSearchStatusProps) {
  function handleToggle() {
    if (enabled) {
      onDisable();
      return;
    }
    onEnable();
  }

  return (
    <div
      className="config-surface flex items-center justify-between gap-2 rounded-md px-2.5 py-1.5"
      title="Tavily 联网搜索，需 API Key"
    >
      <span className="truncate text-[11px] text-fg-secondary">Web 搜索</span>
      <button
        type="button"
        role="switch"
        aria-checked={enabled}
        aria-label={enabled ? "关闭 Web 搜索" : "开启 Web 搜索"}
        className={`relative inline-flex h-5 w-9 shrink-0 items-center rounded-full border transition-colors ${
          enabled
            ? "border-emerald-600/50 bg-emerald-600/90"
            : "border-border-subtle bg-muted"
        }`}
        onClick={handleToggle}
      >
        <span
          aria-hidden
          className={`pointer-events-none inline-block h-3.5 w-3.5 rounded-full bg-white shadow-sm transition-transform ${
            enabled ? "translate-x-[18px]" : "translate-x-0.5"
          }`}
        />
      </button>
    </div>
  );
}
