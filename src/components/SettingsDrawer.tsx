import { useEffect, useMemo, useState } from "react";
import packageJson from "../../package.json";
import { useUpdateProgress } from "../hooks/useUpdateProgress";
import {
  configuredBalanceProviders,
  fetchProviderBalances,
  type ProviderBalanceRow,
} from "../lib/providerBalance";
import {
  checkForAppUpdates,
  fetchLatestReleaseVersion,
  isNewerVersion,
} from "../lib/updater";
import { providerLabel } from "../types";

interface SettingsDrawerProps {
  open: boolean;
  onClose: () => void;
  apiKeyStatus: Record<string, boolean>;
}

function formatLatestVersion(
  loading: boolean,
  latestVersion: string | null,
): string {
  if (loading) return "…";
  return latestVersion ?? "—";
}

function balanceDisplayForProvider(
  provider: string,
  loading: boolean,
  balances: ProviderBalanceRow[],
): string {
  if (loading) return "…";
  return balances.find((row) => row.provider === provider)?.display ?? "—";
}

export function SettingsDrawer({ open, onClose, apiKeyStatus }: SettingsDrawerProps) {
  const currentVersion = packageJson.version;
  const [latestVersion, setLatestVersion] = useState<string | null>(null);
  const [loadingLatest, setLoadingLatest] = useState(false);
  const [updatePending, setUpdatePending] = useState(false);
  const updateProgress = useUpdateProgress();
  const updating = updatePending || updateProgress.phase !== "idle";
  const [balances, setBalances] = useState<ProviderBalanceRow[]>([]);
  const [loadingBalances, setLoadingBalances] = useState(false);

  const configuredProviders = useMemo(
    () => configuredBalanceProviders(apiKeyStatus),
    [apiKeyStatus],
  );

  const hasUpdate =
    latestVersion !== null && isNewerVersion(latestVersion, currentVersion);

  useEffect(() => {
    if (!open) return;

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") onClose();
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [open, onClose]);

  useEffect(() => {
    if (!open) return;

    let cancelled = false;
    setLoadingLatest(true);
    setLatestVersion(null);

    void fetchLatestReleaseVersion()
      .then((version) => {
        if (!cancelled) setLatestVersion(version);
      })
      .finally(() => {
        if (!cancelled) setLoadingLatest(false);
      });

    return () => {
      cancelled = true;
    };
  }, [open]);

  useEffect(() => {
    if (!open) return;
    if (configuredProviders.length === 0) {
      setBalances([]);
      setLoadingBalances(false);
      return;
    }

    let cancelled = false;
    setLoadingBalances(true);
    setBalances([]);

    void fetchProviderBalances(configuredProviders)
      .then((rows) => {
        if (!cancelled) setBalances(rows);
      })
      .finally(() => {
        if (!cancelled) setLoadingBalances(false);
      });

    return () => {
      cancelled = true;
    };
  }, [open, configuredProviders]);

  async function handleUpdate() {
    if (updating || !hasUpdate) return;
    setUpdatePending(true);
    try {
      await checkForAppUpdates("manual");
    } catch (error) {
      console.error(error);
    } finally {
      setUpdatePending(false);
    }
  }

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex justify-end">
      <button
        type="button"
        aria-label="关闭设置"
        className="absolute inset-0 bg-black/35"
        onClick={onClose}
      />
      <aside
        role="dialog"
        aria-modal="true"
        aria-labelledby="settings-drawer-title"
        className="panel relative flex h-full w-80 shrink-0 flex-col gap-4 border-l border-border p-4 shadow-xl"
      >
        <div className="flex items-center justify-between">
          <h2 id="settings-drawer-title" className="text-sm font-semibold text-fg">
            设置
          </h2>
          <button
            type="button"
            className="rounded-md border border-border-subtle px-2 py-1 text-xs text-fg-secondary hover:border-border-hover hover:text-fg"
            onClick={onClose}
          >
            关闭
          </button>
        </div>

        <section className="config-surface rounded-md p-3 text-xs text-fg-secondary">
          <div className="flex items-center justify-between gap-3">
            <span>当前版本</span>
            <span className="text-fg">{currentVersion}</span>
          </div>
          <div className="mt-2 flex items-center justify-between gap-3">
            <span>最新版本</span>
            <span className="text-fg">{formatLatestVersion(loadingLatest, latestVersion)}</span>
          </div>
          {hasUpdate && (
            <button
              type="button"
              className="mt-3 w-full rounded-md border border-border-subtle px-2.5 py-1.5 text-xs text-fg-secondary hover:border-border-hover hover:text-fg disabled:cursor-not-allowed disabled:opacity-60"
              onClick={() => void handleUpdate()}
              disabled={updating}
            >
              {updating ? "更新中…" : "更新"}
            </button>
          )}
        </section>

        {(configuredProviders.length > 0 || loadingBalances) && (
          <section className="config-surface rounded-md p-3 text-xs text-fg-secondary">
            <div className="text-fg">账户余额</div>
            {configuredProviders.map((provider) => (
              <div key={provider} className="mt-2 flex items-center justify-between gap-3">
                <span>{providerLabel(provider)}</span>
                <span className="text-fg">
                  {balanceDisplayForProvider(provider, loadingBalances, balances)}
                </span>
              </div>
            ))}
          </section>
        )}
      </aside>
    </div>
  );
}
