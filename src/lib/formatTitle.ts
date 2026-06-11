const MARKDOWN_INLINE_RE = /\*\*|__|~~|`/g;
const MARKDOWN_LINK_RE = /\[([^\]]*)\]\([^)]*\)/g;

/** 兼容存量标题：去除残留的行内 Markdown 标记 */
export function plainSessionTitle(title: string): string {
  return title.replace(MARKDOWN_INLINE_RE, "").replace(MARKDOWN_LINK_RE, "$1");
}
