import { splitPath } from "./pathUtils";

export function isOoxmlWorkDirSegment(name: string): boolean {
  const lower = name.toLowerCase();
  return lower === "unpacked" || lower.endsWith("_unpacked");
}

export function isIgnoredMentionPath(path: string): boolean {
  return splitPath(path).some(isOoxmlWorkDirSegment);
}

export function sameStringArrays(a: string[], b: string[]): boolean {
  return a.length === b.length && a.every((value, index) => value === b[index]);
}

export function mergeProjectFilePaths(prev: string[], added: string[]): string[] {
  const set = new Set(prev);
  for (const raw of added) {
    const path = raw.trim().replace(/\\/g, "/");
    if (!path || isIgnoredMentionPath(path)) continue;
    set.add(path);
  }
  return [...set].sort();
}
