import { fuzzyMatch, type FuzzyMatch } from "./fuzzy";
import { parentPath, pathBasename } from "./pathUtils";
import type { MentionFileEntry } from "./projectFiles";

export interface FileMentionMatch extends FuzzyMatch {
  isDir: boolean;
  basename: string;
  parentPath: string;
}

export interface MentionFileGroup {
  id: string;
  label: string;
  items: FileMentionMatch[];
}

export interface MentionBrowseContext {
  scopeDir: string | null;
  term: string;
}

export function parseMentionBrowseContext(query: string): MentionBrowseContext {
  const trimmed = query.trim();
  const slashIdx = trimmed.lastIndexOf("/");
  if (slashIdx >= 0) {
    return {
      scopeDir: trimmed.slice(0, slashIdx),
      term: trimmed.slice(slashIdx + 1),
    };
  }
  return { scopeDir: null, term: trimmed };
}

function entryToMatch(
  entry: MentionFileEntry,
  positions: number[] = [],
  score = 0,
): FileMentionMatch {
  return {
    item: entry.path,
    score,
    positions,
    isDir: entry.isDir,
    basename: pathBasename(entry.path),
    parentPath: parentPath(entry.path),
  };
}

function sortBrowseEntries(entries: MentionFileEntry[]): MentionFileEntry[] {
  return [...entries].sort(
    (a, b) =>
      Number(b.isDir) - Number(a.isDir) ||
      b.modifiedMs - a.modifiedMs ||
      a.path.localeCompare(b.path),
  );
}

function directChildren(entries: MentionFileEntry[], dir: string): MentionFileEntry[] {
  const prefix = dir === "." ? "" : `${dir}/`;
  return entries.filter((entry) => {
    if (dir === ".") return !entry.path.includes("/");
    if (!entry.path.startsWith(prefix)) return false;
    const rest = entry.path.slice(prefix.length);
    return rest.length > 0 && !rest.includes("/");
  });
}

function attachEntryMeta(entries: MentionFileEntry[], match: FuzzyMatch): FileMentionMatch {
  const entry = entries.find((item) => item.path === match.item);
  return {
    ...match,
    isDir: entry?.isDir ?? false,
    basename: pathBasename(match.item),
    parentPath: parentPath(match.item),
  };
}

function sortMatchedPaths(
  matches: FuzzyMatch[],
  entries: MentionFileEntry[],
  order: Map<string, number>,
): FileMentionMatch[] {
  return [...matches]
    .sort((a, b) => {
      const entryA = entries.find((entry) => entry.path === a.item);
      const entryB = entries.find((entry) => entry.path === b.item);
      if (entryA?.isDir !== entryB?.isDir) {
        return Number(entryB?.isDir) - Number(entryA?.isDir);
      }
      return (order.get(a.item) ?? 0) - (order.get(b.item) ?? 0);
    })
    .map((match) => attachEntryMeta(entries, match));
}

/** 按浏览上下文过滤 @ 候选；空 @ 仅根级，含 `/` 则进入对应目录 */
export function searchMentionFiles(
  query: string,
  entries: MentionFileEntry[],
): FileMentionMatch[] {
  if (entries.length === 0) return [];

  const ctx = parseMentionBrowseContext(query);
  const order = new Map(entries.map((entry, index) => [entry.path, index]));

  if (ctx.scopeDir !== null) {
    const children = sortBrowseEntries(directChildren(entries, ctx.scopeDir));
    if (!ctx.term) {
      return children.map((entry) => entryToMatch(entry));
    }
    const childNames = children.map((entry) => ({
      entry,
      name: pathBasename(entry.path),
    }));
    const matched = fuzzyMatch(
      ctx.term,
      childNames.map((child) => child.name),
    );
    const pathMatches = matched.map((match) => {
      const child = childNames.find((item) => item.name === match.item);
      return { ...match, item: child!.entry.path };
    });
    return sortMatchedPaths(pathMatches, entries, order);
  }

  if (!ctx.term) {
    return sortBrowseEntries(directChildren(entries, ".")).map((entry) => entryToMatch(entry));
  }

  const matched = fuzzyMatch(
    ctx.term,
    entries.map((entry) => entry.path),
  );
  return sortMatchedPaths(matched, entries, order);
}

function sortGroupItems(items: FileMentionMatch[]): FileMentionMatch[] {
  return [...items].sort(
    (a, b) =>
      Number(b.isDir) - Number(a.isDir) || a.basename.localeCompare(b.basename, undefined, { numeric: true }),
  );
}

/** 全局搜索按父目录分组；目录浏览与根列表保持单组 */
export function groupMentionFileMatches(
  matches: FileMentionMatch[],
  ctx: MentionBrowseContext,
): MentionFileGroup[] {
  if (matches.length === 0) return [];

  if (ctx.scopeDir !== null) {
    return [{ id: ctx.scopeDir, label: ctx.scopeDir, items: matches }];
  }

  if (!ctx.term) {
    return [{ id: ".", label: "项目根目录", items: matches }];
  }

  const groups = new Map<string, FileMentionMatch[]>();
  for (const match of matches) {
    const bucket = groups.get(match.parentPath) ?? [];
    bucket.push(match);
    groups.set(match.parentPath, bucket);
  }

  return [...groups.entries()]
    .sort(([a], [b]) => {
      if (a === ".") return -1;
      if (b === ".") return 1;
      return a.localeCompare(b);
    })
    .map(([parent, items]) => ({
      id: parent,
      label: parent === "." ? "根目录" : `${parent}/`,
      items: sortGroupItems(items),
    }));
}

/** 将全路径上的高亮位置映射到 basename */
export function highlightBasenamePositions(match: FileMentionMatch): number[] {
  if (match.positions.length === 0) return [];
  const baseStart = match.item.length - match.basename.length;
  return match.positions.filter((pos) => pos >= baseStart).map((pos) => pos - baseStart);
}

export function flattenMentionFileGroups(groups: MentionFileGroup[]): FileMentionMatch[] {
  return groups.flatMap((group) => group.items);
}

/** 与 FileMentionPopup 渲染顺序一致的扁平列表，供键盘选择使用 */
export function orderMentionFileMatchesForDisplay(
  matches: FileMentionMatch[],
  query: string,
): FileMentionMatch[] {
  const ctx = parseMentionBrowseContext(query);
  return flattenMentionFileGroups(groupMentionFileMatches(matches, ctx));
}
