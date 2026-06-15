import { invoke } from "@tauri-apps/api/core";

export interface ProviderBalanceRow {
  provider: string;
  display: string;
}

export const BALANCE_UNAVAILABLE = "—";

export const BALANCE_PROVIDERS = ["deepseek", "kimi"] as const;

export type BalanceProvider = (typeof BALANCE_PROVIDERS)[number];

export function configuredBalanceProviders(
  apiKeyStatus: Record<string, boolean>,
): BalanceProvider[] {
  return BALANCE_PROVIDERS.filter((provider) => Boolean(apiKeyStatus[provider]));
}

export function unavailableBalanceRows(providers: BalanceProvider[]): ProviderBalanceRow[] {
  return providers.map((provider) => ({
    provider,
    display: BALANCE_UNAVAILABLE,
  }));
}

export async function fetchProviderBalances(
  providers: BalanceProvider[],
): Promise<ProviderBalanceRow[]> {
  if (providers.length === 0) return [];

  try {
    return await invoke<ProviderBalanceRow[]>("fetch_provider_balances");
  } catch {
    return unavailableBalanceRows(providers);
  }
}
