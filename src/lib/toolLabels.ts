export const TOOL_LABELS: Record<string, string> = {
  fs_list: "列出目录",
  fs_read: "读取文件",
  fs_write: "写入文件",
  fs_search: "搜索文件",
  office_read_to_markdown: "读取文档",
  office_convert: "转换 Office 格式",
  word_create: "创建 Word",
  excel_read: "读取 Excel",
  excel_write: "写入 Excel",
  skill_run: "运行 Skill",
};

export function toolLabel(name: string): string {
  return TOOL_LABELS[name] ?? name;
}

export function formatToolArgs(args: unknown): string {
  if (args === null || args === undefined) {
    return "{}";
  }
  if (typeof args === "object" && !Array.isArray(args)) {
    const entries = Object.entries(args as Record<string, unknown>);
    if (entries.length === 1) {
      const [key, value] = entries[0];
      if (typeof value === "string") {
        return `${key}: ${value}`;
      }
    }
  }
  return JSON.stringify(args, null, 2);
}
