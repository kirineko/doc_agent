interface InitCapsuleProps {
  onInit: () => void;
}

/** 空会话底部轻量入口：点击后阅读文档并生成 starter 推荐问 */
export function InitCapsule({ onInit }: InitCapsuleProps) {
  return (
    <button
      type="button"
      className="inline-flex max-w-full items-center gap-1.5 rounded-full border border-slate-700/70 bg-slate-950/50 px-3 py-1 text-xs text-slate-400 transition hover:border-indigo-600/50 hover:bg-indigo-950/30 hover:text-indigo-200"
      onClick={onInit}
    >
      <span aria-hidden className="text-[10px] text-indigo-400/90">
        ✦
      </span>
      <span className="truncate">根据文档生成推荐问</span>
    </button>
  );
}

export function InitLoadingCapsule() {
  return (
    <span className="inline-flex max-w-full items-center gap-1.5 rounded-full border border-slate-800 bg-slate-950/40 px-3 py-1 text-xs text-slate-500">
      <span className="h-1.5 w-1.5 shrink-0 animate-pulse rounded-full bg-indigo-400" />
      <span className="truncate">正在分析项目文档…</span>
    </span>
  );
}
