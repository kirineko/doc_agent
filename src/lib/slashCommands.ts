export type SlashCategory = "command" | "general" | "word" | "ppt" | "excel" | "pdf" | "web";

export interface SlashTemplate {
  kind: "template";
  id: string;
  category: SlashCategory;
  label: string;
  description: string;
  keywords: string[];
  prompt: string;
}

export interface SlashCommandEntry {
  kind: "command";
  id: string;
  category: SlashCategory;
  label: string;
  description: string;
  keywords: string[];
  acceptsTail?: boolean;
}

export type SlashEntry = SlashTemplate | SlashCommandEntry;

/** @deprecated Use SlashEntry */
export type SlashCommand = SlashEntry;

export const CATEGORY_ORDER: SlashCategory[] = [
  "command",
  "general",
  "word",
  "ppt",
  "excel",
  "pdf",
  "web",
];

export const CATEGORY_LABELS: Record<SlashCategory, string> = {
  command: "命令",
  general: "通用",
  word: "Word",
  ppt: "PPT",
  excel: "Excel",
  pdf: "PDF",
  web: "Web",
};

type SlashTemplateSeed = Omit<SlashTemplate, "kind">;

/** 用户需替换的部分使用 {{提示文字}} 占位符 */
const SLASH_TEMPLATE_SEEDS: SlashTemplateSeed[] = [
  {
    id: "read",
    category: "general",
    label: "阅读分析",
    description: "阅读项目内 Word/PPT/Excel/PDF 等文件",
    keywords: ["阅读", "分析", "概括", "pdf", "docx", "xlsx"],
    prompt: "请阅读 {{文件名}}，概括内容结构；若是表格则总结关键数据，并给改进建议。",
  },
  {
    id: "clarify",
    category: "general",
    label: "先澄清需求",
    description: "尚未确定文档类型或内容时先澄清",
    keywords: ["澄清", "需求", "格式"],
    prompt: "我想做一份文档但还没想清楚，请先帮我澄清需求和格式。",
  },
  {
    id: "search",
    category: "general",
    label: "搜索项目",
    description: "在项目目录中搜索关键词",
    keywords: ["搜索", "查找", "项目"],
    prompt: "请在项目里搜索「{{关键词}}」，告诉我出现在哪些文件。",
  },
  {
    id: "convert",
    category: "general",
    label: "旧格式转换",
    description: "将 .doc/.xls/.ppt 转为新格式",
    keywords: ["转换", "doc", "xls", "ppt", "旧格式"],
    prompt: "请把 {{旧格式文件}} 转成新 Office 格式，并说明是否有格式损失。",
  },
  {
    id: "web-search",
    category: "general",
    label: "联网查资料",
    description: "搜索网络信息并整理要点（需 Tavily Key）",
    keywords: ["联网", "搜索", "资料"],
    prompt: "请联网搜索「{{关键词}}」的最新信息，整理成文档要点。",
  },
  {
    id: "word:create",
    category: "word",
    label: "新建 Word",
    description: "从零生成 .docx 文档",
    keywords: ["word", "docx", "新建", "报告"],
    prompt: "帮我新建一份 Word 文档，主题是{{主题}}。请先简单澄清再开始制作。",
  },
  {
    id: "word:edit",
    category: "word",
    label: "精准修改 Word",
    description: "OOXML 解包改 XML 再回包",
    keywords: ["word", "修改", "xml", "精准", "docx"],
    prompt:
      "请精准修改 {{文件名.docx}}：{{改动说明}}。先解包改 XML 再回包，保留原有排版。",
  },
  {
    id: "word:comment",
    category: "word",
    label: "添加批注",
    description: "在 docx 上添加批注（OOXML 流程）",
    keywords: ["批注", "comment", "审阅"],
    prompt: "请给 {{文件名.docx}} 添加批注：{{批注内容}}。需要的话锚定到对应段落。",
  },
  {
    id: "word:clean-revisions",
    category: "word",
    label: "接受修订",
    description: "接受全部修订并另存干净版",
    keywords: ["修订", "接受", "track changes"],
    prompt: "请接受 {{文件名.docx}} 的全部修订，另存一份干净版本。",
  },
  {
    id: "word:extract-table",
    category: "word",
    label: "提取表格",
    description: "从 Word 提取表格为 CSV",
    keywords: ["表格", "提取", "csv"],
    prompt: "请从 {{文件名.docx}} 提取表格为 CSV，并简要说明数据内容。",
  },
  {
    id: "ppt:create",
    category: "ppt",
    label: "新建 PPT",
    description: "从零生成演示文稿",
    keywords: ["ppt", "pptx", "新建", "演示"],
    prompt: "帮我新建一份 PPT，主题是{{主题}}，大约{{页数}}页。请先确认风格再制作。",
  },
  {
    id: "ppt:edit",
    category: "ppt",
    label: "编辑 PPT",
    description: "修改已有 .pptx",
    keywords: ["ppt", "修改", "编辑"],
    prompt: "请修改 {{文件名.pptx}}：{{改动说明}}（如改文案、换图、增删页）。",
  },
  {
    id: "excel:create",
    category: "excel",
    label: "新建 Excel",
    description: "生成带结构的 xlsx",
    keywords: ["excel", "xlsx", "新建", "表格"],
    prompt: "帮我新建一份 Excel，用途是{{用途}}，需要哪些字段和 sheet？",
  },
  {
    id: "excel:clean",
    category: "excel",
    label: "清洗表格",
    description: "规范化不规则表头",
    keywords: ["清洗", "表头", "normalize"],
    prompt: "{{文件名.xlsx}} 表头比较乱，请规范化并说明改了什么。",
  },
  {
    id: "excel:check-formula",
    category: "excel",
    label: "检查公式",
    description: "重算并列出公式错误",
    keywords: ["公式", "检查", "错误"],
    prompt: "请检查 {{文件名.xlsx}} 的公式是否有错误，列出问题位置。",
  },
  {
    id: "excel:analyze",
    category: "excel",
    label: "数据分析",
    description: "SQL 式汇总、对比、排名",
    keywords: ["分析", "数据", "sql", "汇总"],
    prompt: "请对 {{文件名.xlsx}} 做数据分析：{{分析需求}}（如汇总、对比、排名）。",
  },
  {
    id: "pdf:create",
    category: "pdf",
    label: "Typst 新建 PDF",
    description: "选 Typst 模板写 .typ 并编译",
    keywords: ["typst", "pdf", "新建", "排版", "公式"],
    prompt: "请基于 Typst 新建 PDF，主题是{{主题}}。先选模板写 .typ 再编译。",
  },
  {
    id: "pdf:edit-typst",
    category: "pdf",
    label: "修订 Typst PDF",
    description: "修改已有 .typ 并重新编译",
    keywords: ["typst", "修订", "修改", "重编译"],
    prompt: "请修改 {{文件名.typ}}：{{改动说明}}，改完后重新编译为 PDF。",
  },
  {
    id: "pdf:ops",
    category: "pdf",
    label: "PDF 页面操作",
    description: "合并、拆分、旋转或删除页面",
    keywords: ["合并", "拆分", "旋转", "pdf"],
    prompt: "请处理 {{文件名.pdf}}：{{操作说明}}（合并/拆分/旋转/删除指定页）。",
  },
  {
    id: "pdf:forms",
    category: "pdf",
    label: "PDF 表单",
    description: "填写或处理 PDF 表单",
    keywords: ["表单", "form", "pdf"],
    prompt: "请处理 {{文件名.pdf}} 的表单需求：{{需求说明}}。",
  },
  {
    id: "web:report",
    category: "web",
    label: "HTML 报告",
    description: "生成项目内静态网页报告",
    keywords: ["html", "报告", "网页"],
    prompt: "帮我生成一份 HTML 网页报告，主题是{{主题}}，表格和文字要清晰。",
  },
  {
    id: "web:save-pdf",
    category: "web",
    label: "导出 PDF",
    description: "将 HTML 文件或报告目录导出为 PDF",
    keywords: ["html", "pdf", "导出"],
    prompt: "请把 {{HTML路径}}（HTML 文件或报告目录）导出为 PDF。",
  },
];

/** 可执行斜杠命令（Enter 直接发送，非 prompt 模板）；新 command 追加于此 */
const SLASH_COMMAND_ENTRIES: SlashCommandEntry[] = [
  {
    kind: "command",
    id: "init",
    category: "command",
    label: "初始化 AGENTS.md",
    description: "澄清并生成/更新 AGENTS.md",
    keywords: ["init", "agents", "配置", "profile", "偏好", "agents.md"],
    acceptsTail: true,
  },
];

const SLASH_TEMPLATES: SlashTemplate[] = SLASH_TEMPLATE_SEEDS.map((seed) => ({
  ...seed,
  kind: "template" as const,
}));

export const SLASH_COMMANDS: SlashEntry[] = [...SLASH_COMMAND_ENTRIES, ...SLASH_TEMPLATES];

export function isSlashTemplate(entry: SlashEntry): entry is SlashTemplate {
  return entry.kind === "template";
}

export function isSlashCommandEntry(entry: SlashEntry): entry is SlashCommandEntry {
  return entry.kind === "command";
}
