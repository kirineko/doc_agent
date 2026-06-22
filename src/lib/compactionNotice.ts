/** Manual `/compact` invoke — user is waiting for an explicit action. */
export const COMPACTION_MANUAL_IN_PROGRESS_NOTICE = "正在压缩上下文，请稍候…";

/** Auto compaction during an active turn — lighter copy, no "please wait". */
export const COMPACTION_AUTO_IN_PROGRESS_NOTICE = "正在压缩较早的对话历史…";

export function compactionInProgressNotice(trigger: "auto" | "manual"): string {
  return trigger === "manual"
    ? COMPACTION_MANUAL_IN_PROGRESS_NOTICE
    : COMPACTION_AUTO_IN_PROGRESS_NOTICE;
}

export function isCompactionInProgressNotice(
  message: string | null | undefined,
): boolean {
  return (
    message === COMPACTION_MANUAL_IN_PROGRESS_NOTICE ||
    message === COMPACTION_AUTO_IN_PROGRESS_NOTICE
  );
}
