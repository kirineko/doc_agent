import { MODEL_OPTIONS } from "../types";

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
