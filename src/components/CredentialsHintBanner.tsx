interface CredentialsHintBannerProps {
  visible: boolean;
  onOpenCredentials: () => void;
  onDismiss: () => void;
}

export function CredentialsHintBanner({
  visible,
  onOpenCredentials,
  onDismiss,
}: CredentialsHintBannerProps) {
  if (!visible) return null;

  return (
    <div className="flex min-w-0 flex-1 items-center justify-center gap-2 px-2">
      <div className="flex min-w-0 items-center gap-2 rounded-md border border-amber-600/30 bg-amber-600/10 px-2.5 py-1 text-[11px] text-fg-secondary">
        <span className="truncate">尚未配置模型 API Key，发送前需先配置。</span>
        <button
          type="button"
          className="shrink-0 text-accent hover:underline"
          onClick={onOpenCredentials}
        >
          去配置
        </button>
        <button
          type="button"
          className="shrink-0 opacity-70 hover:opacity-100"
          onClick={onDismiss}
          aria-label="关闭提醒"
        >
          ×
        </button>
      </div>
    </div>
  );
}
