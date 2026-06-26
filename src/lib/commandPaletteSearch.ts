import { SLASH_COMMANDS } from "./slashCommands";
import { fuzzyMatch } from "./fuzzy";
import type { Project, Session } from "../types";

export type CommandPaletteGroup =
  | "actions"
  | "projects"
  | "sessions"
  | "commands";

export interface CommandPaletteItem {
  id: string;
  group: CommandPaletteGroup;
  label: string;
  description?: string;
  keywords?: string;
}

export const COMMAND_PALETTE_ACTIONS: CommandPaletteItem[] = [
  {
    id: "action:add-project",
    group: "actions",
    label: "添加项目目录",
    description: "选择文件夹并创建项目",
    keywords: "add project 项目 目录",
  },
  {
    id: "action:new-session",
    group: "actions",
    label: "新建会话",
    description: "在当前项目下创建会话",
    keywords: "new session 新建",
  },
];

const GROUP_ORDER: CommandPaletteGroup[] = ["actions", "projects", "sessions", "commands"];

export function parseSessionPaletteItemId(
  id: string,
): { projectId: string; sessionId: string } | null {
  if (!id.startsWith("session:")) return null;
  const rest = id.slice("session:".length);
  const separator = rest.indexOf(":");
  if (separator < 0) return null;
  return {
    projectId: rest.slice(0, separator),
    sessionId: rest.slice(separator + 1),
  };
}

export function buildCommandPaletteItems(
  projects: Project[],
  sessions: Session[],
): CommandPaletteItem[] {
  const projectNames = new Map(projects.map((project) => [project.id, project.name]));

  const projectItems: CommandPaletteItem[] = projects.map((project) => ({
    id: `project:${project.id}`,
    group: "projects",
    label: project.name,
    description: project.root_path,
    keywords: project.root_path,
  }));

  const sessionItems: CommandPaletteItem[] = sessions.map((session) => ({
    id: `session:${session.project_id}:${session.id}`,
    group: "sessions",
    label: session.title || "未命名会话",
    description: projectNames.get(session.project_id),
    keywords: `${session.title ?? ""} ${projectNames.get(session.project_id) ?? ""}`.trim(),
  }));

  const commandItems: CommandPaletteItem[] = SLASH_COMMANDS.map((entry) => ({
    id: `command:${entry.id}`,
    group: "commands" as const,
    label: entry.label,
    description: `/${entry.id} — ${entry.description}`,
    keywords: `${entry.id} ${entry.label} ${entry.description}`,
  }));

  return [...COMMAND_PALETTE_ACTIONS, ...projectItems, ...sessionItems, ...commandItems];
}

function scoreItem(query: string, item: CommandPaletteItem): number {
  const haystack = `${item.label} ${item.description ?? ""} ${item.keywords ?? ""}`;
  return fuzzyMatch(query, [haystack])[0]?.score ?? -1;
}

export function searchCommandPaletteItems(
  query: string,
  items: CommandPaletteItem[],
): CommandPaletteItem[] {
  const trimmed = query.trim();
  if (!trimmed) {
    return GROUP_ORDER.flatMap((group) => items.filter((item) => item.group === group));
  }

  return items
    .map((item) => ({ item, score: scoreItem(trimmed, item) }))
    .filter(({ score }) => score > 0)
    .sort((a, b) => b.score - a.score || a.item.label.localeCompare(b.item.label))
    .map(({ item }) => item);
}

export function groupCommandPaletteItems(items: CommandPaletteItem[]): {
  group: CommandPaletteGroup;
  label: string;
  items: CommandPaletteItem[];
}[] {
  const labels: Record<CommandPaletteGroup, string> = {
    actions: "操作",
    projects: "项目",
    sessions: "会话",
    commands: "命令",
  };

  return GROUP_ORDER.map((group) => ({
    group,
    label: labels[group],
    items: items.filter((item) => item.group === group),
  })).filter((section) => section.items.length > 0);
}
