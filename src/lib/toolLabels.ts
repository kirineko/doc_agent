export const TOOL_LABELS: Record<string, string> = {
  fs_list: "列出目录",
  fs_read: "读取文件",
  fs_write: "写入文件",
  fs_patch: "局部修改文件",
  fs_search: "搜索文件",
  image_read: "读取图片",
  image_download: "下载图片",
  clarify_ask: "需求澄清",
  office_read_to_markdown: "读取文档",
  office_convert: "转换 Office 格式",
  excel_read: "读取 Excel",
  excel_write: "写入 Excel",
  skill_read: "读取 Skill 指南",
  skill_run: "运行 Skill",
  ooxml_unpack: "解压 OOXML",
  ooxml_pack: "打包 OOXML",
  docx_comment: "添加批注",
  docx_accept_changes: "接受修订",
  docx_extract_table: "提取 Word 表格",
  excel_describe: "侦察 Excel 结构",
  excel_normalize: "清洗 Excel",
  data_query: "SQL 数据查询",
  xlsx_recalc: "重算公式",
  pdf_merge: "合并 PDF",
  pdf_split: "拆分 PDF",
  pdf_rotate: "旋转 PDF",
  pdf_delete_pages: "删除 PDF 页面",
  pdf_render_pages: "渲染 PDF 页面",
  pdf_read: "读取 PDF",
  html_to_pdf: "HTML 导出 PDF",
  typst_to_pdf: "Typst 导出 PDF",
  typst_list_templates: "列出 Typst 模板",
  typst_read_template: "读取 Typst 模板",
  markdown_to_html: "Markdown 转 HTML",
  markdown_list_templates: "列出 Markdown 模板",
  markdown_read_template: "读取 Markdown 模板",
  web_search: "Web 搜索",
  web_extract: "抽取网页正文",
};

export const REGISTERED_TOOL_NAMES = Object.keys(TOOL_LABELS);

export function toolLabel(name: string): string {
  return TOOL_LABELS[name] ?? "未知工具";
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
