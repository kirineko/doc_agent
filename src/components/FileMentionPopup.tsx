import type { ReactNode } from "react";
import type { FuzzyMatch } from "../lib/fuzzy";

interface FileMentionPopupProps {
  matches: FuzzyMatch[];
  selectedIndex: number;
  onPick: (path: string) => void;
}

function highlight(text: string, positions: number[]): ReactNode {
  if (positions.length === 0) return text;
  const set = new Set(positions);
  const parts: ReactNode[] = [];
  for (let i = 0; i < text.length; i++) {
    const ch = text[i]!;
    if (set.has(i)) {
      parts.push(
        <span key={i} className="text-cyan-200">
          {ch}
        </span>,
      );
    } else {
      parts.push(ch);
    }
  }
  return parts;
}

export function FileMentionPopup({
  matches,
  selectedIndex,
  onPick,
}: FileMentionPopupProps) {
  if (matches.length === 0) {
    return (
      <div className="absolute bottom-full left-0 z-20 mb-1 w-full rounded-md border border-slate-700 bg-slate-950 px-2 py-2 text-xs text-slate-500">
        无匹配文件
      </div>
    );
  }

  return (
    <div className="absolute bottom-full left-0 z-20 mb-1 max-h-48 w-full overflow-y-auto rounded-md border border-slate-700 bg-slate-950 py-1 shadow-lg">
      {matches.map((match, index) => (
        <button
          key={match.item}
          type="button"
          className={`flex w-full px-2 py-1.5 text-left text-xs ${
            index === selectedIndex
              ? "bg-indigo-950/60 text-indigo-100"
              : "text-slate-300 hover:bg-slate-900"
          }`}
          onMouseDown={(e) => {
            e.preventDefault();
            onPick(match.item);
          }}
        >
          <span className="truncate">{highlight(match.item, match.positions)}</span>
        </button>
      ))}
    </div>
  );
}
