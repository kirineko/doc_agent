export function joinPath(base: string, name: string): string {
  return base === "." ? name : `${base}/${name}`;
}

export function parentPath(current: string): string {
  const idx = current.lastIndexOf("/");
  return idx < 0 ? "." : current.slice(0, idx);
}

export function pathSegments(currentPath: string): string[] {
  if (currentPath === ".") return [];
  return currentPath.split("/");
}

export function segmentTarget(segments: string[], index: number): string {
  return segments.slice(0, index + 1).join("/");
}

export function pathBasename(path: string): string {
  const normalized = path.replace(/\\/g, "/");
  const idx = normalized.lastIndexOf("/");
  return idx < 0 ? normalized : normalized.slice(idx + 1);
}

export function splitPath(path: string): string[] {
  return path.split(/[/\\]/);
}
