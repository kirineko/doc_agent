import { MODEL_OPTIONS } from "../types";

export type SendBlocker =
  | { kind: "no_project" }
  | { kind: "no_api_key"; provider: string };

export interface SendReadinessInput {
  activeProjectId?: string;
  model: string;
  apiKeyStatus: Record<string, boolean>;
}

function providerForModel(modelId: string): string | undefined {
  return MODEL_OPTIONS.find((m) => m.id === modelId)?.provider;
}

export function getSendBlocker(input: SendReadinessInput): SendBlocker | undefined {
  if (!input.activeProjectId) return { kind: "no_project" };
  const provider = providerForModel(input.model);
  if (provider && !input.apiKeyStatus[provider]) {
    return { kind: "no_api_key", provider };
  }
  return undefined;
}
