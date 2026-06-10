interface SuggestionCardsProps {
  items: string[];
  onPick: (text: string) => void;
}

export function SuggestionCards({ items, onPick }: SuggestionCardsProps) {
  if (items.length === 0) return null;

  return (
    <div className="shrink-0 border-t border-slate-800/80 pt-2">
      <div className="mb-1.5 text-[11px] text-slate-500">推荐问</div>
      <div className="max-h-28 overflow-y-auto overscroll-contain pr-1">
        <div className="flex flex-wrap gap-1.5">
          {items.map((item) => (
            <button
              key={item}
              type="button"
              className="inline-flex max-w-full shrink-0 rounded-full border border-slate-700 bg-slate-950/50 px-2.5 py-1 text-left text-xs leading-snug break-words whitespace-normal text-slate-300 hover:border-cyan-700 hover:text-cyan-200"
              onClick={() => onPick(item)}
            >
              {item}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
