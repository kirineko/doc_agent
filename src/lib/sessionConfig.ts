import { canonicalModelId, MODEL_OPTIONS, type ModelInfo } from "../types";

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

const EFFORTS = new Set(["low", "medium", "high", "max"]);

export function parseSessionConfig(value: unknown): SessionConfig | undefined {
  if (!value || typeof value !== "object") return undefined;
  const record = value as Record<string, unknown>;
  if (typeof record.model !== "string") return undefined;
  if (typeof record.thinking_enabled !== "boolean") return undefined;
  if (typeof record.thinking_effort !== "string" || !EFFORTS.has(record.thinking_effort)) {
    return undefined;
  }
  return {
    model: canonicalModelId(record.model),
    thinking_enabled: record.thinking_enabled,
    thinking_effort: record.thinking_effort,
  };
}

export function defaultsForModel(model: ModelInfo): SessionConfig {
  return {
    model: model.id,
    thinking_enabled: model.default_thinking_enabled,
    thinking_effort: model.default_thinking_effort,
  };
}

export function resolveSessionConfig(
  config: SessionConfig,
  modelIds: Iterable<string>,
): SessionConfig {
  const known = new Set([...modelIds].map(canonicalModelId));
  const model = canonicalModelId(config.model);
  if (known.has(model)) return { ...config, model };
  return { ...DEFAULT_SESSION_CONFIG };
}

export function applyModelPatch(
  prev: SessionConfig,
  patch: Partial<SessionConfig>,
  models: ModelInfo[],
): SessionConfig {
  if (patch.model) {
    const nextModel = canonicalModelId(patch.model);
    if (nextModel !== prev.model) {
      const target = models.find((item) => item.id === nextModel);
      if (target) {
        return { ...defaultsForModel(target), ...patch, model: target.id };
      }
    }
  }
  return { ...prev, ...patch };
}

export function readStoredSessionConfig(modelIds?: Iterable<string>): SessionConfig {
  const fallbackIds = modelIds ?? MODEL_OPTIONS.filter((model) => model.selectable).map((model) => model.id);
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
  const first = models.find((model) => model.provider === provider && model.selectable !== false);
  if (!first) return undefined;
  return defaultsForModel(first);
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
