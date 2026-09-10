import { renderHook, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useAnchorPosition } from "./useAnchorPosition";

function mockTrigger(rect: Partial<DOMRect>) {
  const trigger = document.createElement("button");
  document.body.appendChild(trigger);
  vi.spyOn(trigger, "getBoundingClientRect").mockReturnValue({
    x: 0,
    y: 0,
    top: 0,
    bottom: 0,
    left: 0,
    right: 0,
    width: 0,
    height: 0,
    toJSON: () => ({}),
    ...rect,
  } as DOMRect);
  return trigger;
}

describe("useAnchorPosition", () => {
  afterEach(() => {
    document.body.replaceChildren();
    vi.restoreAllMocks();
  });

  it("widens the panel to minWidth without overflowing the viewport", async () => {
    const trigger = mockTrigger({
      top: 400,
      bottom: 428,
      left: 40,
      right: 240,
      width: 200,
      height: 28,
    });
    const triggerRef = { current: trigger };
    const { result } = renderHook(() => useAnchorPosition(triggerRef, true, { minWidth: 300 }));
    await waitFor(() => {
      expect(result.current?.width).toBe(300);
    });
    expect(result.current?.left).toBe(40);
  });

  it("shifts left when a wide panel would overflow", async () => {
    vi.spyOn(window, "innerWidth", "get").mockReturnValue(360);
    const trigger = mockTrigger({
      top: 80,
      bottom: 108,
      left: 80,
      right: 280,
      width: 200,
      height: 28,
    });
    const triggerRef = { current: trigger };
    const { result } = renderHook(() => useAnchorPosition(triggerRef, true, { minWidth: 300 }));
    await waitFor(() => {
      expect(result.current?.width).toBe(300);
    });
    expect(result.current?.left).toBe(52);
  });
});
