import { firstPlaceholder } from "./promptPlaceholder";

export interface SlashState {
  active: boolean;
  query: string;
  start: number;
  end: number;
}

/** 检测光标前 `/` 命令区域（行首或空白后，/ 与光标之间无空白） */
export function detectSlash(text: string, cursor: number): SlashState | null {
  const before = text.slice(0, cursor);
  const slash = before.lastIndexOf("/");
  if (slash < 0) return null;
  if (slash > 0 && !/\s/.test(before[slash - 1]!)) return null;
  const query = before.slice(slash + 1);
  if (/\s/.test(query)) return null;
  return { active: true, query, start: slash, end: cursor };
}

/** 将 /query 替换为 prompt 模板；若有占位符则选中第一个 */
export function applySlash(
  text: string,
  state: SlashState,
  prompt: string,
): { text: string; cursor: number; selectionEnd: number } {
  const next = `${text.slice(0, state.start)}${prompt}${text.slice(state.end)}`;
  const ph = firstPlaceholder(next, state.start);
  if (ph) {
    return { text: next, cursor: ph.start, selectionEnd: ph.end };
  }
  const cursorPos = state.start + prompt.length;
  return { text: next, cursor: cursorPos, selectionEnd: cursorPos };
}
