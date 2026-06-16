export interface MentionState {
  active: boolean;
  query: string;
  start: number;
  end: number;
}

export interface MentionToken {
  path: string;
  start: number;
  end: number;
}

/** 终止 @ 查询/路径的字符（空白、@、中文标点等；`/` 允许用于路径） */
const MENTION_STOP_CHAR =
  /[\s@，。；：！？、（）【】[\]{}<>'"`~!#$%^&*+=|\\?]/;

function isMentionPathChar(ch: string): boolean {
  if (ch === "/") return true;
  if (MENTION_STOP_CHAR.test(ch)) return false;
  return /[a-zA-Z0-9_.\-\u4e00-\u9fff]/.test(ch);
}

/** 读取 @ 后连续的路径/查询片段（不含中文标点等终止符） */
export function readUnquotedMentionSegment(text: string): string {
  let i = 0;
  while (i < text.length) {
    if (!isMentionPathChar(text[i]!)) break;
    i++;
  }
  return trimGluedSuffixAfterExtension(text.slice(0, i));
}

/** 检测光标前 `@` 提及区域 */
export function detectMention(text: string, cursor: number): MentionState | null {
  const before = text.slice(0, cursor);
  const at = before.lastIndexOf("@");
  if (at < 0) return null;
  const afterAt = before.slice(at + 1);
  const query = readUnquotedMentionSegment(afterAt);
  if (afterAt.length > query.length) return null;
  return { active: true, query, start: at, end: cursor };
}

/** 含空白、引号或解析终止符的路径用双引号包裹（CSV 风格 `""` 转义） */
export function formatMentionPath(path: string): string {
  if (/[\s"]/.test(path)) {
    return `"${path.replace(/"/g, '""')}"`;
  }
  for (const ch of path) {
    if (!isMentionPathChar(ch)) {
      return `"${path.replace(/"/g, '""')}"`;
    }
  }
  return path;
}

/** 解析 `@` 起点的完整提及 token（支持引号路径） */
export function parseMentionTokenAt(text: string, at: number): MentionToken | null {
  if (text[at] !== "@") return null;
  const rest = text.slice(at + 1);
  if (rest.startsWith('"')) {
    let path = "";
    for (let i = 1; i < rest.length; i++) {
      const ch = rest[i]!;
      if (ch === '"') {
        if (rest[i + 1] === '"') {
          path += '"';
          i++;
          continue;
        }
        return { path, start: at, end: at + 1 + i + 1 };
      }
      path += ch;
    }
    return null;
  }
  const raw = readUnquotedMentionSegment(rest);
  return { path: raw, start: at, end: at + 1 + raw.length };
}

/** 扩展名后紧贴中文（无标点）时截断，避免 `@file.docx后续` 吞掉后文 */
function trimGluedSuffixAfterExtension(token: string): string {
  const m = token.match(/^(.+?\.\w{1,8})([\u4e00-\u9fff（(].+)$/);
  return m ? m[1]! : token;
}

/** 将 @query 替换为 @path（含尾随空格）；路径含空白时用引号 */
export function applyMention(
  text: string,
  state: MentionState,
  path: string,
): { text: string; cursor: number } {
  const encoded = formatMentionPath(path);
  const next = `${text.slice(0, state.start)}@${encoded} ${text.slice(state.end)}`;
  const cursor = state.start + encoded.length + 2;
  return { text: next, cursor };
}

/** Tab 进入目录：保留 @ 弹层，将 query 设为 `dir/` */
export function expandMentionDirectory(
  text: string,
  state: MentionState,
  dirPath: string,
): { text: string; cursor: number } {
  const query = `${dirPath}/`;
  const next = `${text.slice(0, state.start)}@${query}${text.slice(state.end)}`;
  const cursor = state.start + 1 + query.length;
  return { text: next, cursor };
}

/** 取消 @ 提及：删除 `@` 与已输入的查询片段 */
export function cancelMentionAtState(
  text: string,
  state: MentionState,
): { text: string; cursor: number } {
  return {
    text: text.slice(0, state.start) + text.slice(state.end),
    cursor: state.start,
  };
}

/**
 * Backspace 时若光标在 @提及 内或紧随其后的分隔空格上，整体删除该提及。
 *  lone `@` 仅删除 `@` 本身，不波及后续正文。
 */
export function deleteMentionBeforeCursor(
  text: string,
  cursor: number,
): { text: string; cursor: number } | null {
  const before = text.slice(0, cursor);
  const at = before.lastIndexOf("@");
  if (at < 0) return null;

  const token = parseMentionTokenAt(text, at);
  if (!token) return null;

  const hasTrailingSpace = text[token.end] === " ";
  const onTrailingSpace = hasTrailingSpace && cursor === token.end + 1;
  const insideToken = cursor > token.start && cursor <= token.end;
  if (!insideToken && !onTrailingSpace) return null;

  const deleteEnd = onTrailingSpace ? token.end + 1 : token.end;
  return {
    text: text.slice(0, token.start) + text.slice(deleteEnd),
    cursor: token.start,
  };
}
