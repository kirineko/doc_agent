import { findModel, MODEL_OPTIONS, type ModelInfo } from "../types";
import { PARALLEL_LIMIT_MESSAGE } from "./sessionRunState";

export type SendBlocker =
  | { kind: "no_project" }
  | { kind: "no_api_key"; provider: string }
  | { kind: "parallel_limit" }
  | { kind: "clarify_pending" }
  | { kind: "unavailable_model"; message: string };

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

export function getSendBlocker(input: SendReadinessInput): SendBlocker | undefined {
  if (!input.activeProjectId) return { kind: "no_project" };
  if (input.parallelAtCapacity) {
    return { kind: "parallel_limit" };
  }
  const catalog = input.models?.length ? input.models : MODEL_OPTIONS;
  const model = findModel(catalog, input.model);
  if (!model) {
    return { kind: "unavailable_model", message: "未知模型无法发送，请新建会话。" };
  }
  if (model.availability === "retired") {
    return {
      kind: "unavailable_model",
      message: "该模型已不可调用。请新建会话选择 MiMo v2.5 Pro。",
    };
  }
  const provider = model.provider;
  if (provider && !input.apiKeyStatus[provider]) {
    return { kind: "no_api_key", provider };
  }
  return undefined;
}
