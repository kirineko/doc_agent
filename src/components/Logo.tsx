export function Logo() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 32 32"
      fill="none"
      className="h-5 w-5 shrink-0"
      role="img"
      aria-label="Doc Agent"
    >
      <rect
        x="5"
        y="4"
        width="18"
        height="24"
        rx="3"
        stroke="var(--accent)"
        strokeWidth="1.75"
        fill="var(--logo-fill)"
      />
      <line
        x1="9"
        y1="11"
        x2="19"
        y2="11"
        stroke="var(--accent)"
        strokeWidth="1.25"
        strokeLinecap="round"
        opacity="0.85"
      />
      <line
        x1="9"
        y1="15"
        x2="17"
        y2="15"
        stroke="var(--accent)"
        strokeWidth="1.25"
        strokeLinecap="round"
        opacity="0.65"
      />
      <line
        x1="9"
        y1="19"
        x2="15"
        y2="19"
        stroke="var(--accent)"
        strokeWidth="1.25"
        strokeLinecap="round"
        opacity="0.45"
      />
      <path
        d="M6 26 Q16 8 27 6"
        stroke="var(--accent)"
        strokeWidth="2"
        strokeLinecap="round"
        fill="none"
      />
      <circle cx="27" cy="6" r="2" fill="var(--accent)" />
    </svg>
  );
}
