interface InitCapsuleProps {
  onInit: () => void;
}

/** 空会话底部轻量入口：点击后阅读文档并生成 starter 推荐问 */
export function InitCapsule({ onInit }: InitCapsuleProps) {
  return (
    <button
      type="button"
      className="chip-surface inline-flex max-w-full items-center gap-1.5 rounded-full px-3 py-1 text-xs transition hover:border-indigo-400 hover:bg-accent-muted hover:text-link"
      onClick={onInit}
    >
      <span aria-hidden className="text-[10px] text-link">
        ✦
      </span>
      <span className="truncate">根据文档生成推荐问</span>
    </button>
  );
}

export function InitLoadingCapsule() {
  return (
    <span className="chip-surface inline-flex max-w-full items-center gap-1.5 rounded-full px-3 py-1 text-xs">
      <span className="h-1.5 w-1.5 shrink-0 animate-pulse rounded-full bg-indigo-400" />
      <span className="truncate">正在分析项目文档…</span>
    </span>
  );
}
