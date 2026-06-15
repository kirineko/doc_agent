import { MODEL_OPTIONS, type ModelInfo } from "../types";

export type SendBlocker =
  | { kind: "no_project" }
  | { kind: "no_api_key"; provider: string };

export interface SendReadinessInput {
  activeProjectId?: string;
  model: string;
  apiKeyStatus: Record<string, boolean>;
  models?: ModelInfo[];
}

function providerForModel(modelId: string, models?: ModelInfo[]): string | undefined {
  if (models?.length) {
    return models.find((m) => m.id === modelId)?.provider;
  }
  return MODEL_OPTIONS.find((m) => m.id === modelId)?.provider;
}

export function getSendBlocker(input: SendReadinessInput): SendBlocker | undefined {
  if (!input.activeProjectId) return { kind: "no_project" };
  const provider = providerForModel(input.model, input.models);
  if (provider && !input.apiKeyStatus[provider]) {
    return { kind: "no_api_key", provider };
  }
  return undefined;
}
