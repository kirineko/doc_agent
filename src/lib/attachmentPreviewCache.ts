const previewCache = new Map<string, string>();

export function attachmentPreviewCacheKey(projectId: string, path: string): string {
  return `${projectId}:${path}`;
}

export function getCachedAttachmentPreview(
  projectId: string,
  path: string,
): string | undefined {
  return previewCache.get(attachmentPreviewCacheKey(projectId, path));
}

export function setCachedAttachmentPreview(
  projectId: string,
  path: string,
  dataUrl: string,
): void {
  previewCache.set(attachmentPreviewCacheKey(projectId, path), dataUrl);
}

/** 测试或切换项目时清空 */
export function clearAttachmentPreviewCache(): void {
  previewCache.clear();
}
