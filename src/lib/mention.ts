export interface MentionState {
  active: boolean;
  query: string;
  start: number;
  end: number;
}

/** 检测光标前 `@` 提及区域（@ 与光标之间无空白） */
export function detectMention(text: string, cursor: number): MentionState | null {
  const before = text.slice(0, cursor);
  const at = before.lastIndexOf("@");
  if (at < 0) return null;
  const query = before.slice(at + 1);
  if (/\s/.test(query)) return null;
  return { active: true, query, start: at, end: cursor };
}

/** 将 @query 替换为 @path（带尾随空格） */
export function applyMention(
  text: string,
  state: MentionState,
  path: string,
): { text: string; cursor: number } {
  const next = `${text.slice(0, state.start)}@${path} ${text.slice(state.end)}`;
  const cursor = state.start + path.length + 2;
  return { text: next, cursor };
}

/**
 * Backspace 时若光标在 @路径 内或紧随其后空格上，整体删除该提及（含可选尾随空格）。
 * 路径范围从 @ 到下一个空白，基于全文而非光标前片段，避免删一半留后缀。
 */
export function deleteMentionBeforeCursor(
  text: string,
  cursor: number,
): { text: string; cursor: number } | null {
  const before = text.slice(0, cursor);
  const at = before.lastIndexOf("@");
  if (at < 0) return null;

  const pathPart = text.slice(at + 1).match(/^[^\s@]*/)?.[0] ?? "";
  if (!pathPart) return null;

  const tokenEnd = at + 1 + pathPart.length;
  const hasTrailingSpace = text[tokenEnd] === " ";
  let deleteEnd =
    hasTrailingSpace && cursor === tokenEnd + 1 ? tokenEnd + 1 : tokenEnd;

  if (cursor <= at || cursor > deleteEnd) return null;

  // 「分析 @文件 后续」→ 删除后保留单个空格
  if (
    hasTrailingSpace &&
    deleteEnd === tokenEnd &&
    at > 0 &&
    text[at - 1] === " "
  ) {
    deleteEnd = tokenEnd + 1;
  }

  return {
    text: text.slice(0, at) + text.slice(deleteEnd),
    cursor: at,
  };
}
