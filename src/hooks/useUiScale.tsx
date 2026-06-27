import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import {
  UI_SCALE_DEFAULT,
  UI_SCALE_STEP,
  applyWebviewZoom,
  readStoredUiScale,
  snapUiScale,
  stepUiScale,
  writeStoredUiScale,
} from "../lib/uiScale";

interface UiScaleContextValue {
  scale: number;
  setScale: (scale: number) => void;
  zoomIn: () => void;
  zoomOut: () => void;
  resetScale: () => void;
}

const UiScaleContext = createContext<UiScaleContextValue | null>(null);

export function UiScaleProvider({ children }: { children: ReactNode }) {
  const [scale, setScaleState] = useState<number>(readStoredUiScale);

  const commitScale = useCallback((next: number) => {
    const snapped = snapUiScale(next);
    setScaleState(snapped);
    writeStoredUiScale(snapped);
    void applyWebviewZoom(snapped);
  }, []);

  const setScale = useCallback(
    (next: number) => {
      commitScale(next);
    },
    [commitScale],
  );

  const zoomIn = useCallback(() => {
    setScaleState((current) => {
      const next = stepUiScale(current, UI_SCALE_STEP);
      writeStoredUiScale(next);
      void applyWebviewZoom(next);
      return next;
    });
  }, []);

  const zoomOut = useCallback(() => {
    setScaleState((current) => {
      const next = stepUiScale(current, -UI_SCALE_STEP);
      writeStoredUiScale(next);
      void applyWebviewZoom(next);
      return next;
    });
  }, []);

  const resetScale = useCallback(() => {
    commitScale(UI_SCALE_DEFAULT);
  }, [commitScale]);

  const value = useMemo(
    () => ({ scale, setScale, zoomIn, zoomOut, resetScale }),
    [scale, setScale, zoomIn, zoomOut, resetScale],
  );

  return <UiScaleContext.Provider value={value}>{children}</UiScaleContext.Provider>;
}

export function useUiScale(): UiScaleContextValue {
  const ctx = useContext(UiScaleContext);
  if (!ctx) {
    throw new Error("useUiScale must be used within UiScaleProvider");
  }
  return ctx;
}
