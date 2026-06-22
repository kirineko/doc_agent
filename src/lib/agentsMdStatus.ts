import type { MentionFileEntry } from "./projectFiles";

export const AGENTS_MD_PATH = "AGENTS.md";

/** Whether project file index has finished at least one fetch for the active project. */
export type AgentsMdStatus = "idle" | "loading" | "missing" | "loaded";

export function hasAgentsMdInEntries(entries: MentionFileEntry[]): boolean {
  return entries.some((entry) => !entry.isDir && entry.path === AGENTS_MD_PATH);
}

export function resolveAgentsMdStatus(
  projectId: string | undefined,
  filesLoaded: boolean,
  fileEntries: MentionFileEntry[],
): AgentsMdStatus {
  if (!projectId) return "idle";
  if (!filesLoaded) return "loading";
  return hasAgentsMdInEntries(fileEntries) ? "loaded" : "missing";
}

export function agentsMdStatusLabel(status: AgentsMdStatus): string {
  switch (status) {
    case "idle":
      return "未选择项目";
    case "loading":
      return "正在扫描项目文件…";
    case "missing":
      return "未配置 AGENTS.md";
    case "loaded":
      return "已加载 AGENTS.md";
  }
}

export function agentsMdStatusHint(status: AgentsMdStatus): string {
  switch (status) {
    case "idle":
      return "选择项目后可查看 AGENTS.md 状态";
    case "loading":
      return "正在索引项目文件，稍候…";
    case "missing":
      return "项目根尚无 AGENTS.md。输入 /init 或在斜杠菜单「命令」中选择。";
    case "loaded":
      return "项目根 AGENTS.md 已就绪，Agent 每轮对话会自动注入其中的配置。";
  }
}
