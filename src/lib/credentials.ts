import { API_PROVIDERS } from "../types";

export function hasAnyLlmKey(apiKeyStatus: Record<string, boolean>): boolean {
  return API_PROVIDERS.some((provider) => Boolean(apiKeyStatus[provider]));
}
