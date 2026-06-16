import { fuzzyMatch } from "./fuzzy";
import {
  CATEGORY_LABELS,
  CATEGORY_ORDER,
  SLASH_COMMANDS,
  type SlashCategory,
  type SlashCommand,
} from "./slashCommands";

export interface SlashCommandMatch {
  command: SlashCommand;
  score: number;
  labelPositions: number[];
}

export interface SlashCommandGroup {
  category: SlashCategory;
  categoryLabel: string;
  items: SlashCommandMatch[];
}

function commandSearchText(command: SlashCommand): string {
  const categoryLabel = CATEGORY_LABELS[command.category];
  return [command.id, command.label, command.description, categoryLabel, ...command.keywords].join(
    " ",
  );
}

function matchCommand(command: SlashCommand, query: string): SlashCommandMatch | null {
  const q = query.trim();
  if (!q) {
    return { command, score: 0, labelPositions: [] };
  }
  const searchText = commandSearchText(command);
  const matches = fuzzyMatch(q, [searchText]);
  const hit = matches[0];
  if (!hit) return null;
  const labelMatches = fuzzyMatch(q, [command.label]);
  return {
    command,
    score: hit.score,
    labelPositions: labelMatches[0]?.positions ?? [],
  };
}

/** 按分类分组；空 query 返回全部，非空 query 做模糊过滤 */
export function searchSlashCommands(query: string): SlashCommandGroup[] {
  const q = query.trim();
  const matched = SLASH_COMMANDS.map((command) => matchCommand(command, q)).filter(
    (item): item is SlashCommandMatch => item !== null,
  );

  const byCategory = new Map<SlashCategory, SlashCommandMatch[]>();
  for (const item of matched) {
    const list = byCategory.get(item.command.category) ?? [];
    list.push(item);
    byCategory.set(item.command.category, list);
  }

  const groups: SlashCommandGroup[] = [];
  for (const category of CATEGORY_ORDER) {
    const items = byCategory.get(category);
    if (!items || items.length === 0) continue;
    items.sort((a, b) => b.score - a.score);
    groups.push({
      category,
      categoryLabel: CATEGORY_LABELS[category],
      items,
    });
  }
  return groups;
}

export function flattenSlashGroups(groups: SlashCommandGroup[]): SlashCommandMatch[] {
  return groups.flatMap((group) => group.items);
}
