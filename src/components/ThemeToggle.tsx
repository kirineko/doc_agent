import { useTheme } from "../hooks/useTheme";

export function ThemeToggle() {
  const { theme, toggleTheme } = useTheme();
  const isLight = theme === "light";

  return (
    <button
      type="button"
      role="switch"
      aria-checked={isLight}
      aria-label={isLight ? "切换到深色模式" : "切换到浅色模式"}
      className="relative ml-auto inline-flex h-7 w-12 shrink-0 items-center rounded-full border border-border-subtle bg-muted p-0.5 transition-colors"
      onClick={toggleTheme}
    >
      <span
        className={`inline-flex h-5 w-5 items-center justify-center rounded-full bg-elevated text-[10px] text-fg-secondary shadow-sm transition-transform ${
          isLight ? "translate-x-5" : "translate-x-0"
        }`}
        aria-hidden
      >
        {isLight ? "☀" : "☽"}
      </span>
    </button>
  );
}
