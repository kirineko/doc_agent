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

export function inferIsDirForMergedPath(
  path: string,
  map: Map<string, MentionFileEntry>,
  added: string[],
): boolean {
  const existing = map.get(path);
  if (existing?.isDir) return true;
  const normalized = path.replace(/\/$/, "");
  if (path.endsWith("/")) return true;
  const prefix = `${normalized}/`;
  for (const entry of map.values()) {
    if (entry.path.startsWith(prefix)) return true;
  }
  for (const raw of added) {
    const other = raw.trim().replace(/\\/g, "/");
    if (other !== normalized && other.startsWith(prefix)) return true;
  }
  return false;
}

function ensureParentDirEntries(
  map: Map<string, MentionFileEntry>,
  path: string,
  now: number,
): void {
  const segments = splitPath(path);
  for (let index = 0; index < segments.length - 1; index += 1) {
    const parent = segments.slice(0, index + 1).join("/");
    if (isIgnoredMentionPath(parent)) continue;
    const existing = map.get(parent);
    if (!existing) {
      map.set(parent, { path: parent, isDir: true, modifiedMs: now });
      continue;
    }
    if (!existing.isDir) {
      map.set(parent, { ...existing, isDir: true });
    }
  }
}

function promoteParentDirs(entries: MentionFileEntry[]): MentionFileEntry[] {
  const byPath = new Map(entries.map((entry) => [entry.path, entry]));
  for (const entry of entries) {
    if (entry.isDir) continue;
    const segments = splitPath(entry.path);
    for (let index = 0; index < segments.length - 1; index += 1) {
      const parent = segments.slice(0, index + 1).join("/");
      const parentEntry = byPath.get(parent);
      if (parentEntry && !parentEntry.isDir) {
        byPath.set(parent, { ...parentEntry, isDir: true });
      }
    }
  }
  return [...byPath.values()];
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
      isDir: inferIsDirForMergedPath(path, map, added) || (existing?.isDir ?? false),
      modifiedMs: existing?.modifiedMs ?? now,
    });
    if (!map.get(path)?.isDir) {
      ensureParentDirEntries(map, path, now);
    }
  }
  return sortMentionFileEntries(promoteParentDirs([...map.values()]));
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
