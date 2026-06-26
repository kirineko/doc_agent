export function greetingForHour(hour: number): string {
  if (hour < 6) return "午夜好，想处理哪份文档？";
  if (hour < 12) return "早上好，想处理哪份文档？";
  if (hour < 18) return "下午好，想处理哪份文档？";
  return "晚上好，想处理哪份文档？";
}

export function composerEmptyGreeting(): string {
  return greetingForHour(new Date().getHours());
}

export function composerWelcomeMessage(hasProject: boolean): string {
  if (!hasProject) return "请选择或添加项目目录，开始处理文档";
  return composerEmptyGreeting();
}

export function shouldCenterComposer(hasProject: boolean, isEmptyLayout: boolean): boolean {
  return !hasProject || isEmptyLayout;
}

export function isComposerEmptyLayout(
  messageCount: number,
  streamingReasoning: string,
  streamingContent: string,
  busy: boolean,
): boolean {
  if (messageCount > 0) return false;
  if (streamingReasoning.trim() || streamingContent.trim()) return false;
  if (busy) return false;
  return true;
}
