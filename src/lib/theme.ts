export const THEME_STORAGE_KEY = "doc-agent-theme";

const HLJS_LINK_ID = "doc-agent-hljs-theme";

export type Theme = "dark" | "light";

export function parseTheme(value: string | null | undefined): Theme {
  return value === "light" ? "light" : "dark";
}

export function readStoredTheme(): Theme {
  try {
    return parseTheme(localStorage.getItem(THEME_STORAGE_KEY));
  } catch {
    return "dark";
  }
}

function loadHighlightStyles(theme: Theme): void {
  const href =
    theme === "dark"
      ? new URL("highlight.js/styles/github-dark.css", import.meta.url).href
      : new URL("highlight.js/styles/github.css", import.meta.url).href;

  let link = document.getElementById(HLJS_LINK_ID) as HTMLLinkElement | null;
  if (!link) {
    link = document.createElement("link");
    link.id = HLJS_LINK_ID;
    link.rel = "stylesheet";
    document.head.appendChild(link);
  }
  if (link.href !== href) {
    link.href = href;
  }
}

export function applyTheme(theme: Theme): void {
  document.documentElement.dataset.theme = theme;
  loadHighlightStyles(theme);
}

export function writeStoredTheme(theme: Theme): void {
  try {
    localStorage.setItem(THEME_STORAGE_KEY, theme);
  } catch {
    // ignore quota / private mode
  }
}
