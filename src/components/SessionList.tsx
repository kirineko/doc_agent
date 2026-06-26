import {
  DndContext,
  KeyboardSensor,
  PointerSensor,
  closestCenter,
  useSensor,
  useSensors,
  type DragEndEvent,
} from "@dnd-kit/core";
import {
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { plainSessionTitle } from "../lib/formatTitle";
import { formatSessionTime } from "../lib/formatTime";
import type { SessionRunStatus } from "../lib/sessionRunState";
import type { Session } from "../types";

interface SessionListProps {
  sessions: Session[];
  activeSessionId?: string;
  sessionRunStatuses?: Record<string, SessionRunStatus>;
  onSelectSession: (sessionId: string) => void;
  onDeleteSession: (sessionId: string) => void;
  onReorderSessions: (activeId: string, overId: string) => void;
  variant?: "card" | "tree";
}

interface SortableSessionItemProps {
  session: Session;
  active: boolean;
  runStatus: SessionRunStatus;
  variant: "card" | "tree";
  onSelectSession: (sessionId: string) => void;
  onDeleteSession: (sessionId: string) => void;
}

function SortableSessionItem({
  session,
  active,
  runStatus,
  variant,
  onSelectSession,
  onDeleteSession,
}: SortableSessionItemProps) {
  const { attributes, listeners, setNodeRef, setActivatorNodeRef, transform, transition, isDragging } =
    useSortable({ id: session.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  };

  const surfaceClass =
    variant === "tree"
      ? active
        ? "nav-session-active bg-hover"
        : "hover:bg-hover"
      : active
        ? "item-session-active"
        : "item-surface";

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={`group relative rounded-md text-xs ${surfaceClass} ${
        variant === "tree" ? "" : "border"
      } ${isDragging ? "z-10 opacity-80 shadow-md" : ""}`}
    >
      <div className="flex items-stretch">
        <button
          type="button"
          ref={setActivatorNodeRef}
          className="shrink-0 cursor-grab touch-none px-1.5 text-fg-muted opacity-60 hover:opacity-100 active:cursor-grabbing"
          aria-label={`拖动排序：${plainSessionTitle(session.title)}`}
          {...attributes}
          {...listeners}
        >
          ⋮⋮
        </button>
        <button
          type="button"
          className="min-w-0 flex-1 px-1 py-1.5 pr-7 text-left"
          onClick={() => onSelectSession(session.id)}
        >
          <div
            className="truncate font-medium"
            title={plainSessionTitle(session.title)}
          >
            {plainSessionTitle(session.title)}
          </div>
          <div className="flex items-center gap-1.5 text-[11px] text-fg-secondary">
            {runStatus === "running" ? (
              <span
                className="inline-flex items-center gap-1 text-sky-400"
                aria-label="执行中"
              >
                <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-sky-400" />
                执行中
              </span>
            ) : runStatus === "stopping" ? (
              <span className="text-amber-400" aria-label="停止中">
                停止中…
              </span>
            ) : null}
            <span>{formatSessionTime(session.updated_at)}</span>
          </div>
        </button>
      </div>
      <button
        type="button"
        className="absolute right-1 top-1/2 -translate-y-1/2 rounded px-1.5 text-fg-muted opacity-0 transition hover:text-rose-400 group-hover:opacity-100"
        title="删除会话"
        aria-label={`删除会话：${plainSessionTitle(session.title)}`}
        onClick={() => onDeleteSession(session.id)}
      >
        ×
      </button>
    </div>
  );
}

export function SessionList({
  sessions,
  activeSessionId,
  sessionRunStatuses = {},
  onSelectSession,
  onDeleteSession,
  onReorderSessions,
  variant = "card",
}: SessionListProps) {
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 4 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );

  function handleDragEnd(event: DragEndEvent) {
    const { active, over } = event;
    if (!over || active.id === over.id) return;
    onReorderSessions(String(active.id), String(over.id));
  }

  return (
    <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEnd}>
      <SortableContext items={sessions.map((item) => item.id)} strategy={verticalListSortingStrategy}>
        <div className="min-h-0 flex-1 space-y-1 overflow-y-auto">
          {sessions.map((session) => (
            <SortableSessionItem
              key={session.id}
              session={session}
              active={session.id === activeSessionId}
              runStatus={sessionRunStatuses[session.id] ?? "idle"}
              variant={variant}
              onSelectSession={onSelectSession}
              onDeleteSession={onDeleteSession}
            />
          ))}
        </div>
      </SortableContext>
    </DndContext>
  );
}
