import { useEffect, useRef, useState } from "react";
import { ProviderKeyRow } from "./ProviderKeyRow";

interface ProviderApiKeyPanelProps {
  provider: string;
  hasKey: boolean;
  highlighted: boolean;
  onStatusChange: (provider: string, has: boolean) => void;
}

export function ProviderApiKeyPanel({
  provider,
  hasKey,
  highlighted,
  onStatusChange,
}: ProviderApiKeyPanelProps) {
  const detailsRef = useRef<HTMLDetailsElement>(null);
  const [expanded, setExpanded] = useState(!hasKey);

  useEffect(() => {
    setExpanded(!hasKey);
  }, [provider, hasKey]);

  useEffect(() => {
    if (!highlighted) return;
    setExpanded(true);
    requestAnimationFrame(() => {
      detailsRef.current?.scrollIntoView({ block: "nearest" });
      document.getElementById(`api-key-${provider}`)?.scrollIntoView({ block: "nearest" });
    });
  }, [highlighted, provider]);

  return (
    <details
      id="sidebar-api-keys"
      ref={detailsRef}
      open={expanded}
      onToggle={(event) => setExpanded(event.currentTarget.open)}
      className={`config-surface rounded-md ${highlighted ? "border-amber-600/60" : ""}`}
    >
      <summary className="flex cursor-pointer list-none items-center justify-between px-2.5 py-2 text-[11px] marker:content-none [&::-webkit-details-marker]:hidden">
        <span className="uppercase tracking-[0.16em] text-fg-secondary">API Key</span>
        <span className={hasKey ? "text-emerald-600" : "text-amber-600"}>
          {hasKey ? "已配置" : "未配置"}
        </span>
      </summary>
      <div className="border-t border-border px-2 pb-2 pt-1.5">
        <ProviderKeyRow
          provider={provider}
          hasKey={hasKey}
          highlighted={highlighted}
          onStatusChange={onStatusChange}
        />
      </div>
    </details>
  );
}
