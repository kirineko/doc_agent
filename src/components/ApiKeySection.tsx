import { useEffect, useRef, useState } from "react";
import { API_PROVIDERS, providerLabel } from "../types";
import { ProviderKeyRow } from "./ProviderKeyRow";

interface ApiKeySectionProps {
  apiKeyStatus: Record<string, boolean>;
  highlightProvider?: string;
  onApiKeyStatusChange: (provider: string, has: boolean) => void;
}

function configuredSummary(apiKeyStatus: Record<string, boolean>): string {
  const configured = API_PROVIDERS.filter((provider) => apiKeyStatus[provider]).map((provider) =>
    providerLabel(provider),
  );
  if (configured.length === 0) return "未配置";
  return configured.join("、");
}

export function ApiKeySection({
  apiKeyStatus,
  highlightProvider,
  onApiKeyStatusChange,
}: ApiKeySectionProps) {
  const hasAnyKey = API_PROVIDERS.some((provider) => apiKeyStatus[provider]);
  const detailsRef = useRef<HTMLDetailsElement>(null);
  const [expanded, setExpanded] = useState(!hasAnyKey);

  useEffect(() => {
    setExpanded(!hasAnyKey);
  }, [hasAnyKey]);

  useEffect(() => {
    if (!highlightProvider) return;
    setExpanded(true);
    requestAnimationFrame(() => {
      detailsRef.current?.scrollIntoView({ block: "nearest" });
      document.getElementById(`api-key-${highlightProvider}`)?.scrollIntoView({ block: "nearest" });
    });
  }, [highlightProvider]);

  return (
    <details
      id="sidebar-api-keys"
      ref={detailsRef}
      open={expanded}
      onToggle={(event) => setExpanded(event.currentTarget.open)}
      className={`config-surface shrink-0 rounded-md ${highlightProvider ? "border-amber-600/60" : ""}`}
    >
      <summary className="flex cursor-pointer list-none items-center justify-between px-2.5 py-2 text-[11px] marker:content-none [&::-webkit-details-marker]:hidden">
        <span className="uppercase tracking-[0.16em] text-fg-secondary">API Key</span>
        <span className={hasAnyKey ? "text-emerald-600" : "text-amber-600"}>
          {configuredSummary(apiKeyStatus)}
        </span>
      </summary>
      <div className="space-y-1.5 border-t border-border px-2 pb-2 pt-1.5">
        {API_PROVIDERS.map((provider) => (
          <ProviderKeyRow
            key={provider}
            provider={provider}
            hasKey={Boolean(apiKeyStatus[provider])}
            highlighted={highlightProvider === provider}
            onStatusChange={onApiKeyStatusChange}
          />
        ))}
      </div>
    </details>
  );
}
