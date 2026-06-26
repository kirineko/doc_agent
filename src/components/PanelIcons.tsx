interface IconProps {
  className?: string;
}

export function FolderOpenIcon({ className = "h-3 w-3" }: IconProps) {
  return (
    <svg
      aria-hidden
      viewBox="0 0 16 16"
      className={className}
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
    >
      <path
        d="M2.5 4.5h4l1 1.5h6v7.5H2.5V4.5z"
        strokeLinejoin="round"
      />
      <path d="M2.5 6.5h11" strokeLinecap="round" />
    </svg>
  );
}

export function FolderIcon({ className = "h-3 w-3" }: IconProps) {
  return (
    <svg
      aria-hidden
      viewBox="0 0 16 16"
      className={className}
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
    >
      <path
        d="M2.5 4.5h4l1 1.5h6v6.5H2.5V4.5z"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export function FileIcon({ className = "h-3 w-3" }: IconProps) {
  return (
    <svg
      aria-hidden
      viewBox="0 0 16 16"
      className={className}
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
    >
      <path
        d="M5 2.5h4.5L12.5 5.5v8H5V2.5z"
        strokeLinejoin="round"
      />
      <path d="M9.5 2.5V5.5H12.5" strokeLinejoin="round" />
    </svg>
  );
}

export function RefreshIcon({ className = "h-3 w-3" }: IconProps) {
  return (
    <svg
      aria-hidden
      viewBox="0 0 16 16"
      className={className}
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
    >
      <path
        d="M11.5 2.5V5H9"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path
        d="M4.5 13.5V11H7"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path
        d="M11.5 5A4.5 4.5 0 0 0 4.2 6.2L4.5 6.5M4.5 11a4.5 4.5 0 0 0 7.3 1.3L11.5 11"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export function ChevronDownIcon({ className = "h-3 w-3" }: IconProps) {
  return (
    <svg
      aria-hidden
      viewBox="0 0 16 16"
      className={className}
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
    >
      <path d="M4 6.5 8 10.5 12 6.5" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

export function ChevronRightIcon({ className = "h-3 w-3" }: IconProps) {
  return (
    <svg
      aria-hidden
      viewBox="0 0 16 16"
      className={className}
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
    >
      <path d="M6.5 4 10.5 8 6.5 12" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

export function SlashIcon({ className = "h-3 w-3" }: IconProps) {
  return (
    <svg
      aria-hidden
      viewBox="0 0 16 16"
      className={className}
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
    >
      <path d="M10.5 3.5 5.5 12.5" strokeLinecap="round" />
    </svg>
  );
}

export function PlusIcon({ className = "h-3 w-3" }: IconProps) {
  return (
    <svg
      aria-hidden
      viewBox="0 0 16 16"
      className={className}
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
    >
      <path d="M8 3.5v9M3.5 8h9" strokeLinecap="round" />
    </svg>
  );
}

export function ImageIcon({ className = "h-3 w-3" }: IconProps) {
  return (
    <svg
      aria-hidden
      viewBox="0 0 16 16"
      className={className}
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
    >
      <rect x="2.5" y="3.5" width="11" height="9" rx="1.25" />
      <circle cx="6" cy="7" r="1.25" />
      <path d="M3.5 11 6.5 8.5 8.5 10l2-1.5 1.5 2.5" strokeLinejoin="round" />
    </svg>
  );
}

const ICON_BTN_SHARED =
  "inline-flex items-center justify-center border border-transparent text-fg-secondary transition hover:bg-hover disabled:cursor-default disabled:opacity-40";

const PANEL_ICON_BTN = `${ICON_BTN_SHARED} min-h-6 min-w-6 rounded hover:text-link`;
const TOOLBAR_ICON_BTN = `${ICON_BTN_SHARED} h-7 w-7 rounded-md hover:text-fg`;

export const TOOLBAR_ICON_CLASS = "h-3.5 w-3.5";

export function panelIconButtonClassName(options?: {
  active?: boolean;
  size?: "panel" | "toolbar";
}): string {
  const base = options?.size === "toolbar" ? TOOLBAR_ICON_BTN : PANEL_ICON_BTN;
  const active = options?.active ? "border-border bg-hover text-fg" : "";
  return `${base} ${active}`.trim();
}
