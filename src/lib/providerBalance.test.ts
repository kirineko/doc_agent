import { describe, expect, it, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import {
  BALANCE_UNAVAILABLE,
  configuredBalanceProviders,
  fetchProviderBalances,
  unavailableBalanceRows,
} from "./providerBalance";

describe("configuredBalanceProviders", () => {
  it("returns only providers with saved keys", () => {
    expect(
      configuredBalanceProviders({
        deepseek: true,
        kimi: false,
        mimo: true,
      }),
    ).toEqual(["deepseek"]);
  });

  it("returns empty list when no balance providers are configured", () => {
    expect(configuredBalanceProviders({})).toEqual([]);
  });
});

describe("unavailableBalanceRows", () => {
  it("builds placeholder rows for each provider", () => {
    expect(unavailableBalanceRows(["deepseek", "kimi"])).toEqual([
      { provider: "deepseek", display: BALANCE_UNAVAILABLE },
      { provider: "kimi", display: BALANCE_UNAVAILABLE },
    ]);
  });
});

describe("fetchProviderBalances", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("skips invoke when no providers are configured", async () => {
    await expect(fetchProviderBalances([])).resolves.toEqual([]);
    expect(invoke).not.toHaveBeenCalled();
  });

  it("returns rows from backend command", async () => {
    vi.mocked(invoke).mockResolvedValue([
      { provider: "deepseek", display: "¥12.34" },
    ]);

    await expect(fetchProviderBalances(["deepseek"])).resolves.toEqual([
      { provider: "deepseek", display: "¥12.34" },
    ]);
    expect(invoke).toHaveBeenCalledWith("fetch_provider_balances");
  });

  it("returns unavailable rows when command fails", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("network"));

    await expect(fetchProviderBalances(["deepseek", "kimi"])).resolves.toEqual([
      { provider: "deepseek", display: BALANCE_UNAVAILABLE },
      { provider: "kimi", display: BALANCE_UNAVAILABLE },
    ]);
  });
});
