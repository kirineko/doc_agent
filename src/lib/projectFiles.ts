import { splitPath } from "./pathUtils";

export interface MentionFileEntry {
  path: string;
  isDir: boolean;
  modifiedMs: number;
}

export function isOoxmlWorkDirSegment(name: string): boolean {
  const lower = name.toLowerCase();
  return lower === "unpacked" || lower.endsWith("_unpacked");
}

export function isIgnoredMentionPath(path: string): boolean {
  return splitPath(path).some(isOoxmlWorkDirSegment);
}

export function sameMentionFileEntries(a: MentionFileEntry[], b: MentionFileEntry[]): boolean {
  return (
    a.length === b.length &&
    a.every((value, index) => {
      const other = b[index];
      return (
        other &&
        value.path === other.path &&
        value.isDir === other.isDir &&
        value.modifiedMs === other.modifiedMs
      );
    })
  );
}

export function sortMentionFileEntries(entries: MentionFileEntry[]): MentionFileEntry[] {
  return [...entries].sort(
    (a, b) => b.modifiedMs - a.modifiedMs || a.path.localeCompare(b.path),
  );
}

export function projectFileEntryFromApi(entry: {
  path: string;
  is_dir: boolean;
  modified_ms: number;
}): MentionFileEntry {
  return {
    path: entry.path,
    isDir: entry.is_dir,
    modifiedMs: entry.modified_ms,
  };
}

export function mergeProjectFileEntries(
  prev: MentionFileEntry[],
  added: string[],
): MentionFileEntry[] {
  const map = new Map(prev.map((entry) => [entry.path, entry]));
  const now = Date.now();
  for (const raw of added) {
    const path = raw.trim().replace(/\\/g, "/");
    if (!path || isIgnoredMentionPath(path)) continue;
    const existing = map.get(path);
    map.set(path, {
      path,
      isDir: existing?.isDir ?? false,
      modifiedMs: existing?.modifiedMs ?? now,
    });
  }
  return sortMentionFileEntries([...map.values()]);
}

/** @deprecated use sameMentionFileEntries */
export function sameStringArrays(a: string[], b: string[]): boolean {
  return a.length === b.length && a.every((value, index) => value === b[index]);
}

/** @deprecated use mergeProjectFileEntries */
export function mergeProjectFilePaths(prev: string[], added: string[]): string[] {
  return mergeProjectFileEntries(
    prev.map((path) => ({ path, isDir: false, modifiedMs: 0 })),
    added,
  ).map((entry) => entry.path);
}
