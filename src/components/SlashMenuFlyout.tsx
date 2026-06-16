import { useEffect, useRef, useState } from "react";
import {
  CATEGORY_LABELS,
  CATEGORY_ORDER,
  SLASH_COMMANDS,
  type SlashCategory,
} from "../lib/slashCommands";
import { SLASH_COMMAND_ROW_CLASS, slashCommandIdClassName } from "../lib/slashCommandRow";

interface SlashMenuFlyoutProps {
  open: boolean;
  onClose: () => void;
  onPick: (commandId: string) => void;
}

export function SlashMenuFlyout({ open, onClose, onPick }: SlashMenuFlyoutProps) {
  const rootRef = useRef<HTMLDivElement>(null);
  const [activeCategory, setActiveCategory] = useState<SlashCategory>("general");

  useEffect(() => {
    if (open) setActiveCategory("general");
  }, [open]);

  useEffect(() => {
    if (!open) return;
    function onPointerDown(event: MouseEvent) {
      if (!rootRef.current?.contains(event.target as Node)) {
        onClose();
      }
    }
    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") onClose();
    }
    document.addEventListener("mousedown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("mousedown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [open, onClose]);

  if (!open) return null;

  const commands = SLASH_COMMANDS.filter((item) => item.category === activeCategory);

  return (
    <div
      ref={rootRef}
      className="mention-popup absolute bottom-full left-0 z-30 mb-1 w-full overflow-hidden rounded-md shadow-lg"
    >
      <div className="flex h-52 flex-col">
        <div className="flex shrink-0 gap-1 overflow-x-auto border-b border-border px-2 py-1.5">
          {CATEGORY_ORDER.map((category) => {
            const selected = category === activeCategory;
            return (
              <button
                key={category}
                type="button"
                className={`shrink-0 rounded-full px-2.5 py-0.5 text-xs transition ${
                  selected
                    ? "mention-item-selected font-medium"
                    : "text-fg-secondary hover:bg-hover hover:text-fg"
                }`}
                onClick={() => setActiveCategory(category)}
              >
                {CATEGORY_LABELS[category]}
              </button>
            );
          })}
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto py-1">
          {commands.map((command) => (
            <button
              key={command.id}
              type="button"
              className={`${SLASH_COMMAND_ROW_CLASS} text-fg hover:bg-hover`}
              onMouseDown={(event) => {
                event.preventDefault();
                onPick(command.id);
                onClose();
              }}
            >
              <span className={slashCommandIdClassName()} title={`/${command.id}`}>
                /{command.id}
              </span>
              <span className="truncate font-medium">{command.label}</span>
              <span className="truncate text-xs text-fg-muted" title={command.description}>
                {command.description}
              </span>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
