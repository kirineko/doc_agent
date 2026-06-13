import { useEffect, useRef } from "react";
import { checkForAppUpdates } from "../lib/updater";

const STARTUP_DELAY_MS = 3000;

export function useAppUpdater() {
  const started = useRef(false);

  useEffect(() => {
    if (started.current) return;
    started.current = true;

    const timer = window.setTimeout(() => {
      void checkForAppUpdates("silent");
    }, STARTUP_DELAY_MS);

    return () => window.clearTimeout(timer);
  }, []);
}
