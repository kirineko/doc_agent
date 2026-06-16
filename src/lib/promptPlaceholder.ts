export interface PlaceholderRange {
  start: number;
  end: number;
  hint: string;
}

const PLACEHOLDER_PATTERN = /\{\{([^}]*)\}\}/g;

/** 查找文本中全部 `{{hint}}` 占位符 */
export function findPlaceholders(text: string): PlaceholderRange[] {
  const ranges: PlaceholderRange[] = [];
  for (const match of text.matchAll(PLACEHOLDER_PATTERN)) {
    const full = match[0];
    const index = match.index;
    if (full === undefined || index === undefined) continue;
    ranges.push({
      start: index,
      end: index + full.length,
      hint: match[1] ?? "",
    });
  }
  return ranges;
}

/** 第一个占位符；可选从 offset 起找（用于斜杠插入后的聚焦） */
export function firstPlaceholder(text: string, fromIndex = 0): PlaceholderRange | null {
  return findPlaceholders(text).find((item) => item.start >= fromIndex) ?? null;
}

/** 光标是否落在占位符内（含右边界，便于 Backspace 整块删除） */
export function placeholderCoveringCursor(
  text: string,
  cursor: number,
): PlaceholderRange | null {
  for (const ph of findPlaceholders(text)) {
    if (cursor > ph.start && cursor <= ph.end) return ph;
  }
  return null;
}

/** Backspace：光标在占位符内部或紧接 `}}` 后时，删除整个占位符 */
export function deletePlaceholderBeforeCursor(
  text: string,
  cursor: number,
): { text: string; cursor: number } | null {
  const ph = placeholderCoveringCursor(text, cursor);
  if (!ph) return null;
  return {
    text: text.slice(0, ph.start) + text.slice(ph.end),
    cursor: ph.start,
  };
}

/** Delete：光标在占位符起点时，删除整个占位符 */
export function deletePlaceholderAtCursor(
  text: string,
  cursor: number,
): { text: string; cursor: number } | null {
  for (const ph of findPlaceholders(text)) {
    if (cursor === ph.start) {
      return {
        text: text.slice(0, ph.start) + text.slice(ph.end),
        cursor: ph.start,
      };
    }
  }
  return null;
}
