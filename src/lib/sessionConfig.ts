import { MODEL_OPTIONS, type ModelInfo } from "../types";

export interface SessionConfig {
  model: string;
  thinking_enabled: boolean;
  thinking_effort: string;
}

export const DEFAULT_SESSION_CONFIG: SessionConfig = {
  model: MODEL_OPTIONS[0].id,
  thinking_enabled: true,
  thinking_effort: "high",
};

export const SESSION_CONFIG_STORAGE_KEY = "doc-agent-last-session-config";

export function parseSessionConfig(value: unknown): SessionConfig | undefined {
  if (!value || typeof value !== "object") return undefined;
  const record = value as Record<string, unknown>;
  if (typeof record.model !== "string") return undefined;
  if (typeof record.thinking_enabled !== "boolean") return undefined;
  if (record.thinking_effort !== "high" && record.thinking_effort !== "max") return undefined;
  return {
    model: record.model,
    thinking_enabled: record.thinking_enabled,
    thinking_effort: record.thinking_effort,
  };
}

export function resolveSessionConfig(
  config: SessionConfig,
  modelIds: Iterable<string>,
): SessionConfig {
  const known = new Set(modelIds);
  if (known.has(config.model)) return config;
  return { ...DEFAULT_SESSION_CONFIG };
}

export function readStoredSessionConfig(modelIds?: Iterable<string>): SessionConfig {
  const fallbackIds = modelIds ?? MODEL_OPTIONS.map((model) => model.id);
  try {
    const raw = localStorage.getItem(SESSION_CONFIG_STORAGE_KEY);
    if (!raw) return DEFAULT_SESSION_CONFIG;
    const parsed = parseSessionConfig(JSON.parse(raw));
    if (!parsed) return DEFAULT_SESSION_CONFIG;
    return resolveSessionConfig(parsed, fallbackIds);
  } catch {
    return DEFAULT_SESSION_CONFIG;
  }
}

export function writeStoredSessionConfig(config: SessionConfig): void {
  try {
    localStorage.setItem(SESSION_CONFIG_STORAGE_KEY, JSON.stringify(config));
  } catch {
    // ignore quota / private mode
  }
}

export function sessionConfigFromSession(session: {
  model: string;
  thinking_enabled: boolean;
  thinking_effort: string;
}): SessionConfig {
  return {
    model: session.model,
    thinking_enabled: session.thinking_enabled,
    thinking_effort: session.thinking_effort,
  };
}

export function configForProviderFirstModel(
  models: ModelInfo[],
  provider: string,
): Partial<SessionConfig> | undefined {
  const first = models.find((model) => model.provider === provider);
  if (!first) return undefined;
  return {
    model: first.id,
    thinking_enabled: true,
    thinking_effort: "high",
  };
}

export function isSessionModelLocked(chatMessageCount: number): boolean {
  return chatMessageCount > 0;
}

export function buildCreateSessionRequest(projectId: string, config: SessionConfig) {
  return {
    project_id: projectId,
    title: "新会话",
    model: config.model,
    thinking_enabled: config.thinking_enabled,
    thinking_effort: config.thinking_effort,
  };
}
