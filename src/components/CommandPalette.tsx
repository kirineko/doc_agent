import { useEffect, useMemo, useRef, useState } from "react";
import {
  buildCommandPaletteItems,
  groupCommandPaletteItems,
  searchCommandPaletteItems,
  type CommandPaletteItem,
} from "../lib/commandPaletteSearch";
import type { Project, Session } from "../types";

interface CommandPaletteProps {
  open: boolean;
  projects: Project[];
  sessions: Session[];
  onClose: () => void;
  /** Return `false` to keep the palette open (e.g. action blocked). */
  onSelectItem: (item: CommandPaletteItem) => boolean | void;
}

function shouldClosePalette(result: boolean | void): boolean {
  return result !== false;
}

function applyPaletteSelection(
  item: CommandPaletteItem,
  onSelectItem: (item: CommandPaletteItem) => boolean | void,
  onClose: () => void,
): void {
  if (shouldClosePalette(onSelectItem(item))) {
    onClose();
  }
}

export function CommandPalette({
  open,
  projects,
  sessions,
  onClose,
  onSelectItem,
}: CommandPaletteProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const selectedItemRef = useRef<HTMLButtonElement>(null);
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);

  const allItems = useMemo(
    () => buildCommandPaletteItems(projects, sessions),
    [projects, sessions],
  );

  const filteredItems = useMemo(
    () => searchCommandPaletteItems(query, allItems),
    [query, allItems],
  );

  const grouped = useMemo(() => groupCommandPaletteItems(filteredItems), [filteredItems]);

  const flatItems = useMemo(
    () => grouped.flatMap((section) => section.items),
    [grouped],
  );

  const flatItemsRef = useRef(flatItems);
  flatItemsRef.current = flatItems;
  const selectedIndexRef = useRef(selectedIndex);
  selectedIndexRef.current = selectedIndex;
  const onCloseRef = useRef(onClose);
  onCloseRef.current = onClose;
  const onSelectItemRef = useRef(onSelectItem);
  onSelectItemRef.current = onSelectItem;

  useEffect(() => {
    if (!open) {
      setQuery("");
      setSelectedIndex(0);
      return;
    }
    inputRef.current?.focus();
  }, [open]);

  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  useEffect(() => {
    if (!open || flatItems.length === 0) return;
    selectedItemRef.current?.scrollIntoView({ block: "nearest" });
  }, [open, selectedIndex, flatItems]);

  useEffect(() => {
    if (!open) return;

    function handleKeyDown(event: KeyboardEvent) {
      const items = flatItemsRef.current;
      const index = selectedIndexRef.current;

      if (event.key === "Escape") {
        event.preventDefault();
        onCloseRef.current();
        return;
      }
      if (event.key === "ArrowDown") {
        event.preventDefault();
        setSelectedIndex((current) => Math.min(current + 1, Math.max(items.length - 1, 0)));
        return;
      }
      if (event.key === "ArrowUp") {
        event.preventDefault();
        setSelectedIndex((current) => Math.max(current - 1, 0));
        return;
      }
      if (event.key === "Enter" && items[index]) {
        event.preventDefault();
        applyPaletteSelection(items[index], onSelectItemRef.current, onCloseRef.current);
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [open]);

  if (!open) return null;

  let runningIndex = 0;

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/20 pt-[12vh] backdrop-blur-[1px]"
      onMouseDown={onClose}
    >
      <div
        className="w-full max-w-lg rounded-xl border border-border-subtle bg-elevated shadow-xl"
        onMouseDown={(event) => event.stopPropagation()}
        role="dialog"
        aria-label="命令面板"
      >
        <input
          ref={inputRef}
          type="text"
          className="w-full border-0 border-b border-border-subtle bg-transparent px-4 py-3 text-sm outline-none"
          placeholder="搜索项目、会话、命令…"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
        />
        <div className="max-h-80 overflow-y-auto py-2">
          {flatItems.length === 0 ? (
            <div className="px-4 py-6 text-center text-sm text-fg-muted">无匹配结果</div>
          ) : (
            grouped.map((section) => (
              <div key={section.group} className="px-2 py-1">
                <div className="px-2 py-1 text-[10px] font-medium uppercase tracking-wide text-fg-muted">
                  {section.label}
                </div>
                {section.items.map((item) => {
                  const itemIndex = runningIndex++;
                  const selected = itemIndex === selectedIndex;
                  return (
                    <button
                      key={item.id}
                      ref={selected ? selectedItemRef : undefined}
                      type="button"
                      className={`flex w-full flex-col rounded-md px-3 py-2 text-left ${
                        selected ? "bg-hover" : "hover:bg-hover"
                      }`}
                      onMouseEnter={() => setSelectedIndex(itemIndex)}
                      onClick={() => applyPaletteSelection(item, onSelectItem, onClose)}
                    >
                      <span className="text-sm text-fg">{item.label}</span>
                      {item.description && (
                        <span className="truncate text-xs text-fg-muted">{item.description}</span>
                      )}
                    </button>
                  );
                })}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
