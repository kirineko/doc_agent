import { invoke } from "@tauri-apps/api/core";
import { MODEL_OPTIONS, type ModelInfo } from "../types";

let cached: ModelInfo[] | null = null;

export async function loadModels(): Promise<ModelInfo[]> {
  if (cached) return cached;
  const models = await invoke<ModelInfo[]>("list_models");
  cached = models;
  return models;
}

export function providerForModel(models: ModelInfo[], modelId: string): string | undefined {
  const fromCatalog = models.find((m) => m.id === modelId);
  if (fromCatalog) return fromCatalog.provider;
  return MODEL_OPTIONS.find((m) => m.id === modelId)?.provider;
}

export function modelSupportsVision(models: ModelInfo[], modelId: string): boolean {
  const fromCatalog = models.find((m) => m.id === modelId);
  if (fromCatalog) return fromCatalog.supports_vision;
  return MODEL_OPTIONS.find((m) => m.id === modelId)?.supportsVision ?? false;
}
