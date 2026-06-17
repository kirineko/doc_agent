import { describe, expect, it } from "vitest";
import { getSendBlocker } from "./sendReadiness";

describe("getSendBlocker", () => {
  it("blocks when no project selected", () => {
    expect(
      getSendBlocker({
        model: "deepseek-v4-flash",
        apiKeyStatus: { deepseek: true },
      }),
    ).toEqual({ kind: "no_project" });
  });

  it("blocks when model provider key missing", () => {
    expect(
      getSendBlocker({
        activeProjectId: "p1",
        model: "kimi-k2.6",
        apiKeyStatus: { deepseek: true, kimi: false },
      }),
    ).toEqual({ kind: "no_api_key", provider: "kimi" });
  });

  it("allows send when project and key are ready", () => {
    expect(
      getSendBlocker({
        activeProjectId: "p1",
        model: "deepseek-v4-flash",
        apiKeyStatus: { deepseek: true },
      }),
    ).toBeUndefined();
  });

  it("blocks when parallel capacity is reached", () => {
    expect(
      getSendBlocker({
        activeProjectId: "p1",
        model: "deepseek-v4-flash",
        apiKeyStatus: { deepseek: true },
        parallelAtCapacity: true,
      }),
    ).toEqual({ kind: "parallel_limit" });
  });
});
