interface CredentialsButtonProps {
  showStatusDot: boolean;
  onClick: () => void;
}

export function CredentialsButton({ showStatusDot, onClick }: CredentialsButtonProps) {
  return (
    <button
      type="button"
      aria-label="打开密钥与服务"
      title="密钥与服务"
      className="relative inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-md border border-border-subtle text-xs text-fg-secondary transition hover:border-border-hover hover:text-fg"
      onClick={onClick}
    >
      <svg
        aria-hidden
        viewBox="0 0 16 16"
        className="h-3.5 w-3.5"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
      >
        <circle cx="5.5" cy="5.5" r="3" />
        <path d="M8.5 8.5 12 12" strokeLinecap="round" />
        <path d="M10 6.5h2.5M11.25 5.25v2.5" strokeLinecap="round" />
      </svg>
      {showStatusDot && (
        <span
          aria-hidden
          className="absolute right-0.5 top-0.5 h-1.5 w-1.5 rounded-full bg-amber-500"
        />
      )}
    </button>
  );
}
