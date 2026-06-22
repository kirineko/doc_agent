export type FileBusyError = {
  error: "file_busy";
  path: string;
  message?: string;
  blocking_session_id?: string;
};

export function parseFileBusyError(raw: string): FileBusyError | undefined {
  const trimmed = raw.trim();
  if (!trimmed.startsWith("{")) return undefined;
  try {
    const parsed = JSON.parse(trimmed) as Record<string, unknown>;
    if (parsed.error !== "file_busy" || typeof parsed.path !== "string") {
      return undefined;
    }
    return {
      error: "file_busy",
      path: parsed.path,
      message: typeof parsed.message === "string" ? parsed.message : undefined,
      blocking_session_id:
        typeof parsed.blocking_session_id === "string"
          ? parsed.blocking_session_id
          : undefined,
    };
  } catch {
    return undefined;
  }
}

export function formatFileBusyMessage(
  error: FileBusyError,
  blockingSessionTitle?: string,
): string {
  const who =
    blockingSessionTitle?.trim() ||
    (error.blocking_session_id ? `会话 ${error.blocking_session_id}` : "其他会话");
  if (error.message?.trim()) return error.message.trim();
  return `文件 ${error.path} 已被${who}占用，请稍后重试。`;
}

export function isFileBusySummary(summary: string): boolean {
  return parseFileBusyError(summary) !== undefined;
}

/** 将 tool_result.summary 格式化为用户可读的错误文案 */
export function formatToolResultError(summary: string): string {
  const fileBusy = parseFileBusyError(summary);
  if (fileBusy) return formatFileBusyMessage(fileBusy);

  const trimmed = summary.trim();
  if (trimmed.startsWith("{")) {
    try {
      const parsed = JSON.parse(trimmed) as Record<string, unknown>;
      if (typeof parsed.message === "string" && parsed.message.trim()) {
        return parsed.message.trim();
      }
      if (typeof parsed.error === "string") {
        return parsed.error;
      }
    } catch {
      // fall through
    }
  }

  return trimmed.length > 240 ? `${trimmed.slice(0, 240)}…` : trimmed;
}
