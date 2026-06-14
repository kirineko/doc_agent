import type { Session } from "../types";

export const SESSION_ORDER_STORAGE_KEY = "doc-agent-session-order";

type SessionOrderStore = Record<string, string[]>;

function readStore(): SessionOrderStore {
  try {
    const raw = localStorage.getItem(SESSION_ORDER_STORAGE_KEY);
    if (!raw) return {};
    const parsed: unknown = JSON.parse(raw);
    if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
      return {};
    }
    const store: SessionOrderStore = {};
    for (const [projectId, order] of Object.entries(parsed)) {
      if (Array.isArray(order) && order.every((id) => typeof id === "string")) {
        store[projectId] = order;
      }
    }
    return store;
  } catch {
    return {};
  }
}

function writeStore(store: SessionOrderStore): void {
  try {
    localStorage.setItem(SESSION_ORDER_STORAGE_KEY, JSON.stringify(store));
  } catch {
    // ignore quota / private mode
  }
}

export function hasManualOrder(projectId: string): boolean {
  return Object.prototype.hasOwnProperty.call(readStore(), projectId);
}

export function readProjectOrder(projectId: string): string[] | null {
  if (!hasManualOrder(projectId)) return null;
  return readStore()[projectId] ?? [];
}

export function writeProjectOrder(projectId: string, orderIds: string[]): void {
  const store = readStore();
  store[projectId] = orderIds;
  writeStore(store);
}

export function clearProjectOrder(projectId: string): void {
  const store = readStore();
  if (!Object.prototype.hasOwnProperty.call(store, projectId)) return;
  delete store[projectId];
  writeStore(store);
}

export function applySessionOrder(sessions: Session[], orderIds: string[]): Session[] {
  const byId = new Map(sessions.map((session) => [session.id, session]));
  const ordered: Session[] = [];
  const seen = new Set<string>();

  for (const id of orderIds) {
    const session = byId.get(id);
    if (!session) continue;
    ordered.push(session);
    seen.add(id);
  }

  const unknown: Session[] = [];
  for (const session of sessions) {
    if (!seen.has(session.id)) unknown.push(session);
  }
  unknown.sort((a, b) => b.updated_at.localeCompare(a.updated_at));

  return [...unknown, ...ordered];
}

export function displaySessionsForProject(sessions: Session[], projectId: string): Session[] {
  const order = readProjectOrder(projectId);
  if (!order) return sessions;
  return applySessionOrder(sessions, order);
}

/** 在列表中移动 activeId 到 overId 位置；id 不存在时返回 null。 */
export function moveSessionInList(
  sessions: Session[],
  activeId: string,
  overId: string,
): Session[] | null {
  const oldIndex = sessions.findIndex((item) => item.id === activeId);
  const newIndex = sessions.findIndex((item) => item.id === overId);
  if (oldIndex < 0 || newIndex < 0) return null;

  const next = [...sessions];
  const [moved] = next.splice(oldIndex, 1);
  next.splice(newIndex, 0, moved);
  return next;
}

export function prependSessionToOrder(projectId: string, sessionId: string): void {
  if (!hasManualOrder(projectId)) return;
  const existing = readProjectOrder(projectId) ?? [];
  writeProjectOrder(projectId, [sessionId, ...existing.filter((id) => id !== sessionId)]);
}

export function removeSessionFromOrder(projectId: string, sessionId: string): void {
  if (!hasManualOrder(projectId)) return;
  const existing = readProjectOrder(projectId) ?? [];
  const next = existing.filter((id) => id !== sessionId);
  if (next.length === 0) {
    clearProjectOrder(projectId);
    return;
  }
  writeProjectOrder(projectId, next);
}
