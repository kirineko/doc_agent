import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { LiveToolCall } from "../components/ToolChainPanel";
import {
  DEFAULT_INSPECTOR_TAB,
  isActiveToolStatus,
  resolveInspectorAutoSwitch,
  type InspectorTab,
} from "../lib/inspectorTab";

interface UseInspectorTabOptions {
  liveTools: LiveToolCall[];
  turnNonce: number;
  activeSessionId?: string;
}

export function useInspectorTab({
  liveTools,
  turnNonce,
  activeSessionId,
}: UseInspectorTabOptions) {
  const [tab, setTabState] = useState<InspectorTab>(DEFAULT_INSPECTOR_TAB);
  const userPinnedRef = useRef<InspectorTab | null>(null);
  const autoSwitchedRef = useRef(false);

  const hasActiveTool = useMemo(
    () => liveTools.some((item) => isActiveToolStatus(item.status)),
    [liveTools],
  );

  const setTab = useCallback((next: InspectorTab, manual = false) => {
    if (manual) {
      userPinnedRef.current = next;
    }
    setTabState(next);
  }, []);

  useEffect(() => {
    userPinnedRef.current = null;
    autoSwitchedRef.current = false;
  }, [turnNonce, activeSessionId]);

  useEffect(() => {
    const next = resolveInspectorAutoSwitch(
      tab,
      userPinnedRef.current,
      hasActiveTool,
      autoSwitchedRef.current,
    );
    if (!next) return;
    autoSwitchedRef.current = true;
    setTabState(next);
  }, [hasActiveTool, tab, activeSessionId]);

  return { tab, setTab };
}
