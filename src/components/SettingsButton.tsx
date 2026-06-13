interface SettingsButtonProps {
  onClick: () => void;
}

export function SettingsButton({ onClick }: SettingsButtonProps) {
  return (
    <button
      type="button"
      aria-label="打开设置"
      title="设置"
      className="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-md border border-border-subtle text-xs text-fg-secondary transition hover:border-border-hover hover:text-fg"
      onClick={onClick}
    >
      <span aria-hidden>⚙</span>
    </button>
  );
}
