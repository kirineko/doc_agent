import { MODEL_OPTIONS, type ModelInfo } from "../types";
import { PARALLEL_LIMIT_MESSAGE } from "./sessionRunState";

export type SendBlocker =
  | { kind: "no_project" }
  | { kind: "no_api_key"; provider: string }
  | { kind: "parallel_limit" }
  | { kind: "clarify_pending" };

export interface SendReadinessInput {
  activeProjectId?: string;
  model: string;
  apiKeyStatus: Record<string, boolean>;
  models?: ModelInfo[];
  parallelAtCapacity?: boolean;
}

export { PARALLEL_LIMIT_MESSAGE };

export function isParallelLimitError(error: unknown): boolean {
  return String(error).includes(PARALLEL_LIMIT_MESSAGE);
}

function providerForModel(modelId: string, models?: ModelInfo[]): string | undefined {
  if (models?.length) {
    return models.find((m) => m.id === modelId)?.provider;
  }
  return MODEL_OPTIONS.find((m) => m.id === modelId)?.provider;
}

export function getSendBlocker(input: SendReadinessInput): SendBlocker | undefined {
  if (!input.activeProjectId) return { kind: "no_project" };
  if (input.parallelAtCapacity) {
    return { kind: "parallel_limit" };
  }
  const provider = providerForModel(input.model, input.models);
  if (provider && !input.apiKeyStatus[provider]) {
    return { kind: "no_api_key", provider };
  }
  return undefined;
}
