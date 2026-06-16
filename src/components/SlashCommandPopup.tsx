import { useEffect, useRef, type ReactNode } from "react";
import type { SlashCommandGroup } from "../lib/slashFuzzy";

interface SlashCommandPopupProps {
  groups: SlashCommandGroup[];
  selectedIndex: number;
  onPick: (commandId: string) => void;
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

export function SlashCommandPopup({
  groups,
  selectedIndex,
  onPick,
}: SlashCommandPopupProps) {
  const listRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const container = listRef.current;
    if (!container) return;
    const selected = container.querySelector<HTMLElement>('[data-slash-selected="true"]');
    selected?.scrollIntoView({ block: "nearest" });
  }, [selectedIndex, groups]);

  if (groups.length === 0) {
    return (
      <div className="mention-popup absolute bottom-full left-0 z-20 mb-1 w-full rounded-md px-2 py-2 text-xs text-fg-muted">
        无匹配命令，试试 word、pdf、read
      </div>
    );
  }

  let runningIndex = 0;

  return (
    <div
      ref={listRef}
      className="mention-popup absolute bottom-full left-0 z-20 mb-1 max-h-48 w-full overflow-y-auto rounded-md py-1 shadow-lg"
    >
      {groups.map((group) => (
        <div key={group.category}>
          <div className="px-2 py-0.5 text-[11px] font-medium text-fg-muted">
            {group.categoryLabel}
          </div>
          {group.items.map((match) => {
            const index = runningIndex++;
            const selected = index === selectedIndex;
            return (
              <button
                key={match.command.id}
                type="button"
                data-slash-selected={selected ? "true" : undefined}
                className={`flex w-full min-w-0 items-center gap-1.5 px-2 py-1 text-left text-xs ${
                  selected ? "mention-item-selected" : "text-fg hover:bg-hover"
                }`}
                onMouseDown={(e) => {
                  e.preventDefault();
                  onPick(match.command.id);
                }}
              >
                <span className="shrink-0 font-mono text-[11px] text-fg-muted">
                  /{match.command.id}
                </span>
                <span className="shrink-0 font-medium">
                  {highlight(match.command.label, match.labelPositions)}
                </span>
                <span
                  className={`min-w-0 flex-1 truncate text-[11px] ${
                    selected ? "text-fg-secondary" : "text-fg-muted"
                  }`}
                  title={match.command.description}
                >
                  {match.command.description}
                </span>
              </button>
            );
          })}
        </div>
      ))}
    </div>
  );
}
