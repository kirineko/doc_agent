/** Shared row layout for slash command id / label / description columns. */
export const SLASH_COMMAND_ROW_CLASS =
  "grid w-full grid-cols-[10rem_minmax(7.5rem,auto)_minmax(0,1fr)] items-center gap-x-2 px-2 py-1 text-left text-xs";

export function slashCommandIdClassName(): string {
  return "min-w-0 truncate font-mono text-xs text-fg-muted";
}
