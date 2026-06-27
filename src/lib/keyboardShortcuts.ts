const IS_MAC =
  typeof navigator !== "undefined" && /Mac/i.test(navigator.platform);

export function isMacPlatform(): boolean {
  return IS_MAC;
}

export function modKeyLabel(): string {
  return IS_MAC ? "⌘" : "Ctrl+";
}

export function formatShortcut(key: string, options?: { shift?: boolean }): string {
  const letter = key.toUpperCase();
  if (IS_MAC) {
    return options?.shift ? `⌘⇧${letter}` : `⌘${letter}`;
  }
  return options?.shift ? `${modKeyLabel()}Shift+${letter}` : `${modKeyLabel()}${letter}`;
}

export function isModShortcut(event: KeyboardEvent, key: string): boolean {
  const mod = IS_MAC ? event.metaKey : event.ctrlKey;
  return mod && !event.shiftKey && !event.altKey && event.key.toLowerCase() === key.toLowerCase();
}

export function isCommandPaletteShortcut(event: KeyboardEvent): boolean {
  return isModShortcut(event, "k");
}

export function isNewSessionShortcut(event: KeyboardEvent): boolean {
  return isModShortcut(event, "n");
}

export const ADD_PROJECT_SHORTCUT_KEY = "o";

export function isAddProjectShortcut(event: KeyboardEvent): boolean {
  return isModShortcut(event, ADD_PROJECT_SHORTCUT_KEY);
}

function hasModKey(event: KeyboardEvent): boolean {
  return IS_MAC ? event.metaKey : event.ctrlKey;
}

export function isUiScaleShortcutBlocked(event: KeyboardEvent): boolean {
  return event.isComposing || event.keyCode === 229;
}

export function isZoomInShortcut(event: KeyboardEvent): boolean {
  if (!hasModKey(event) || event.altKey) return false;
  return event.key === "=" || event.key === "+" || event.key === "Add";
}

export function isZoomOutShortcut(event: KeyboardEvent): boolean {
  if (!hasModKey(event) || event.altKey) return false;
  return event.key === "-" || event.key === "_";
}

export function isZoomResetShortcut(event: KeyboardEvent): boolean {
  if (!hasModKey(event) || event.shiftKey || event.altKey) return false;
  return event.key === "0";
}
