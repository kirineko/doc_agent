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
        <span key={i} className="text-link">
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
      <div className="mention-popup absolute bottom-full left-0 z-20 mb-1 w-full rounded-md px-2 py-2 text-xs text-fg-muted">
        无匹配文件
      </div>
    );
  }

  return (
    <div className="mention-popup absolute bottom-full left-0 z-20 mb-1 max-h-48 w-full overflow-y-auto rounded-md py-1 shadow-lg">
      {matches.map((match, index) => (
        <button
          key={match.item}
          type="button"
          className={`flex w-full px-2 py-1.5 text-left text-xs ${
            index === selectedIndex
              ? "mention-item-selected"
              : "text-fg hover:bg-hover"
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
