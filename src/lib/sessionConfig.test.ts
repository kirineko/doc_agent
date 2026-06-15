import { beforeEach, describe, expect, it } from "vitest";
import {
  configForProviderFirstModel,
  DEFAULT_SESSION_CONFIG,
  parseSessionConfig,
  readStoredSessionConfig,
  writeStoredSessionConfig,
  isSessionModelLocked,
} from "./sessionConfig";
import type { ModelInfo } from "../types";

const MODELS: ModelInfo[] = [
  {
    id: "deepseek-v4-flash",
    label: "DeepSeek V4 Flash",
    provider: "deepseek",
    api_model: "deepseek-v4-flash",
    supports_vision: false,
    supports_effort: true,
    max_context: 100000,
  },
  {
    id: "kimi-k2.6",
    label: "Kimi K2.6",
    provider: "kimi",
    api_model: "kimi-k2.6",
    supports_vision: true,
    supports_effort: false,
    max_context: 100000,
  },
];

describe("session config persistence", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("falls back to default when storage is empty", () => {
    expect(readStoredSessionConfig()).toEqual(DEFAULT_SESSION_CONFIG);
  });

  it("reads and writes last config", () => {
    writeStoredSessionConfig({
      model: "kimi-k2.6",
      thinking_enabled: false,
      thinking_effort: "max",
    });
    expect(readStoredSessionConfig()).toEqual({
      model: "kimi-k2.6",
      thinking_enabled: false,
      thinking_effort: "max",
    });
  });

  it("falls back when stored model is unknown", () => {
    writeStoredSessionConfig({
      model: "removed-model",
      thinking_enabled: true,
      thinking_effort: "high",
    });
    expect(readStoredSessionConfig(MODELS.map((model) => model.id))).toEqual(DEFAULT_SESSION_CONFIG);
  });

  it("rejects invalid stored payload", () => {
    localStorage.setItem("doc-agent-last-session-config", JSON.stringify({ model: "kimi-k2.6" }));
    expect(readStoredSessionConfig()).toEqual(DEFAULT_SESSION_CONFIG);
    expect(parseSessionConfig({ model: "kimi-k2.6", thinking_enabled: true, thinking_effort: "high" })).toEqual({
      model: "kimi-k2.6",
      thinking_enabled: true,
      thinking_effort: "high",
    });
  });
});

describe("configForProviderFirstModel", () => {
  it("selects first model with thinking defaults", () => {
    expect(configForProviderFirstModel(MODELS, "deepseek")).toEqual({
      model: "deepseek-v4-flash",
      thinking_enabled: true,
      thinking_effort: "high",
    });
  });
});

describe("isSessionModelLocked", () => {
  it("unlocks empty session", () => {
    expect(isSessionModelLocked(0)).toBe(false);
  });

  it("locks after first chat message", () => {
    expect(isSessionModelLocked(1)).toBe(true);
  });
});
