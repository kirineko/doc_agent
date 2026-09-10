import { invoke } from "@tauri-apps/api/core";
import { findModel, MODEL_OPTIONS, type ModelInfo } from "../types";

let cached: ModelInfo[] | null = null;

export async function loadModels(): Promise<ModelInfo[]> {
  if (cached) return cached;
  const models = await invoke<ModelInfo[]>("list_models");
  cached = models;
  return models;
}

export function providerForModel(models: ModelInfo[], modelId: string): string | undefined {
  return findModel(models, modelId)?.provider;
}

export function modelSupportsVision(models: ModelInfo[], modelId: string): boolean {
  return findModel(models, modelId)?.supports_vision ?? false;
}

export function modelAvailability(models: ModelInfo[], modelId: string): string | undefined {
  return findModel(models, modelId)?.availability;
}

export function visionModelHint(models: ModelInfo[]): string {
  const names = (models.length ? models : MODEL_OPTIONS)
    .filter((model) => model.selectable && model.supports_vision)
    .map((model) => model.label);
  return `当前模型不支持图片输入，请选用 ${names.join("、")}`;
}
