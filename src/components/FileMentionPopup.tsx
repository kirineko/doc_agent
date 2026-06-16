import { useEffect, useRef, type ReactNode } from "react";
import {
  groupMentionFileMatches,
  highlightBasenamePositions,
  parseMentionBrowseContext,
  type FileMentionMatch,
} from "../lib/mentionFiles";

interface FileMentionPopupProps {
  query: string;
  matches: FileMentionMatch[];
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

function BrowseBreadcrumb({ scopeDir }: { scopeDir: string }) {
  const segments = scopeDir.split("/").filter(Boolean);
  return (
    <div className="flex min-w-0 items-center gap-0.5 truncate border-b border-border-subtle px-2 py-1 text-xs text-fg-muted">
      <span className="shrink-0" aria-hidden>
        ⌂
      </span>
      {segments.map((segment, index) => (
        <span key={`${index}-${segment}`} className="flex min-w-0 items-center gap-0.5">
          <span className="shrink-0">/</span>
          <span className={index === segments.length - 1 ? "truncate text-fg-secondary" : "truncate"}>
            {segment}
          </span>
        </span>
      ))}
    </div>
  );
}

function MentionRow({
  match,
  selected,
  showParentPath,
  onPick,
}: {
  match: FileMentionMatch;
  selected: boolean;
  showParentPath: boolean;
  onPick: (path: string) => void;
}) {
  const label = match.isDir ? `${match.basename}/` : match.basename;
  const positions = highlightBasenamePositions(match);

  return (
    <button
      type="button"
      data-mention-selected={selected ? "true" : undefined}
      className={`flex w-full min-w-0 items-center gap-1.5 py-1 pl-2 pr-2 text-left text-xs ${
        selected ? "mention-item-selected" : "text-fg hover:bg-hover"
      }`}
      onMouseDown={(e) => {
        e.preventDefault();
        onPick(match.item);
      }}
    >
      <span className="w-3.5 shrink-0 text-center text-xs text-fg-muted" aria-hidden>
        {match.isDir ? "📁" : "📄"}
      </span>
      <span className="min-w-0 shrink truncate font-medium">{highlight(label, positions)}</span>
      {showParentPath && match.parentPath !== "." && (
        <span
          className={`min-w-0 flex-1 truncate text-xs ${
            selected ? "text-fg-secondary" : "text-fg-muted"
          }`}
          title={match.parentPath}
        >
          {match.parentPath}
        </span>
      )}
    </button>
  );
}

export function FileMentionPopup({
  query,
  matches,
  selectedIndex,
  onPick,
}: FileMentionPopupProps) {
  const listRef = useRef<HTMLDivElement>(null);
  const browseContext = parseMentionBrowseContext(query);
  const groups = groupMentionFileMatches(matches, browseContext);
  const browsing = browseContext.scopeDir !== null;
  const searching = Boolean(browseContext.term) && !browsing;
  const showGroupHeaders = searching && groups.length > 1;

  useEffect(() => {
    const container = listRef.current;
    if (!container) return;
    const selected = container.querySelector<HTMLElement>('[data-mention-selected="true"]');
    selected?.scrollIntoView({ block: "nearest" });
  }, [selectedIndex, matches]);

  if (matches.length === 0) {
    const emptyLabel = browsing
      ? `${browseContext.scopeDir}/ 下暂无内容`
      : "无匹配文件";
    return (
      <div className="mention-popup absolute bottom-full left-0 z-20 mb-1 w-full rounded-md px-2 py-2 text-xs text-fg-muted">
        {emptyLabel}
      </div>
    );
  }

  let runningIndex = 0;

  return (
    <div
      ref={listRef}
      className="mention-popup absolute bottom-full left-0 z-20 mb-1 max-h-48 w-full overflow-y-auto rounded-md py-1 shadow-lg"
    >
      {browsing && browseContext.scopeDir && <BrowseBreadcrumb scopeDir={browseContext.scopeDir} />}
      {!browsing && !searching && (
        <div className="px-2 py-0.5 text-xs text-fg-muted">输入名称搜索，Tab/Enter 选中，Esc 关闭</div>
      )}
      {groups.map((group) => (
        <div key={group.id}>
          {showGroupHeaders && (
            <div className="px-2 py-0.5 text-xs font-medium text-fg-muted">{group.label}</div>
          )}
          {!showGroupHeaders && searching && groups.length === 1 && group.id !== "." && (
            <div className="px-2 py-0.5 text-xs font-medium text-fg-muted">{group.label}</div>
          )}
          {group.items.map((match) => {
            const index = runningIndex++;
            return (
              <MentionRow
                key={match.item}
                match={match}
                selected={index === selectedIndex}
                showParentPath={searching}
                onPick={onPick}
              />
            );
          })}
        </div>
      ))}
    </div>
  );
}
