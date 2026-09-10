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

export type ToolResultError = {
  headline: string;
  detail?: string;
  hint?: string;
};

const DETAIL_MAX = 600;
const DETAIL_HEAD = 400;
const DETAIL_TAIL = 200;

function truncateDetail(detail: string): string {
  if (detail.length <= DETAIL_MAX) return detail;
  return `${detail.slice(0, DETAIL_HEAD)}…${detail.slice(-DETAIL_TAIL)}`;
}

function truncatePlain(text: string): string {
  return text.length > 240 ? `${text.slice(0, 240)}…` : text;
}

/** 解析 tool_result.summary 为标题 / 详情 / 提示。 */
export function parseToolResultError(summary: string): ToolResultError {
  const fileBusy = parseFileBusyError(summary);
  if (fileBusy) {
    return { headline: formatFileBusyMessage(fileBusy) };
  }

  const trimmed = summary.trim();
  if (trimmed.startsWith("{")) {
    try {
      const parsed = JSON.parse(trimmed) as Record<string, unknown>;
      const error =
        typeof parsed.error === "string" && parsed.error.trim()
          ? parsed.error.trim()
          : undefined;
      const message =
        typeof parsed.message === "string" && parsed.message.trim()
          ? parsed.message.trim()
          : undefined;
      const headline = error ?? message ?? truncatePlain(trimmed);
      const detail =
        typeof parsed.detail === "string" && parsed.detail
          ? truncateDetail(parsed.detail)
          : undefined;
      const hint =
        typeof parsed.hint === "string" && parsed.hint.trim()
          ? parsed.hint.trim()
          : undefined;
      return { headline, detail, hint };
    } catch {
      // fall through
    }
  }

  return { headline: truncatePlain(trimmed) };
}

/** 将 tool_result.summary 格式化为用户可读的错误文案 */
export function formatToolResultError(summary: string): string {
  return parseToolResultError(summary).headline;
}
