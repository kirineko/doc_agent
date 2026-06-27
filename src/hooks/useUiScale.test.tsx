import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { UI_SCALE_STORAGE_KEY } from "../lib/uiScale";
import { UiScaleProvider, useUiScale } from "./useUiScale";

vi.mock("../lib/uiScale", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../lib/uiScale")>();
  return {
    ...actual,
    applyWebviewZoom: vi.fn(async () => undefined),
  };
});

import { applyWebviewZoom } from "../lib/uiScale";

function wrapper({ children }: { children: React.ReactNode }) {
  return <UiScaleProvider>{children}</UiScaleProvider>;
}

describe("useUiScale", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.mocked(applyWebviewZoom).mockClear();
  });

  it("defaults to 100% when storage is empty", () => {
    const { result } = renderHook(() => useUiScale(), { wrapper });
    expect(result.current.scale).toBe(1);
  });

  it("setScale snaps and applies", () => {
    const { result } = renderHook(() => useUiScale(), { wrapper });

    act(() => {
      result.current.setScale(1.45);
    });

    expect(result.current.scale).toBe(1.4);
    expect(applyWebviewZoom).toHaveBeenCalledWith(1.4);
    expect(localStorage.getItem(UI_SCALE_STORAGE_KEY)).toBe("1.4");
  });

  it("zoomIn steps by 0.2", () => {
    localStorage.setItem(UI_SCALE_STORAGE_KEY, "1.2");
    const { result } = renderHook(() => useUiScale(), { wrapper });

    act(() => {
      result.current.zoomIn();
    });

    expect(result.current.scale).toBe(1.4);
    expect(applyWebviewZoom).toHaveBeenCalledWith(1.4);
  });

  it("resetScale returns to 100%", () => {
    localStorage.setItem(UI_SCALE_STORAGE_KEY, "1.8");
    const { result } = renderHook(() => useUiScale(), { wrapper });

    act(() => {
      result.current.resetScale();
    });

    expect(result.current.scale).toBe(1);
    expect(applyWebviewZoom).toHaveBeenCalledWith(1);
  });
});
