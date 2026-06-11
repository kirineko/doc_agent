import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import { ThemeTestWrapper } from "../test/renderWithTheme";
import { useTheme } from "./useTheme";

describe("useTheme", () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.removeAttribute("data-theme");
  });

  it("defaults to light theme", () => {
    const { result } = renderHook(() => useTheme(), { wrapper: ThemeTestWrapper });
    expect(result.current.theme).toBe("light");
  });

  it("toggleTheme switches light and dark", () => {
    const { result } = renderHook(() => useTheme(), { wrapper: ThemeTestWrapper });

    act(() => {
      result.current.toggleTheme();
    });
    expect(result.current.theme).toBe("dark");
    expect(localStorage.getItem("doc-agent-theme")).toBe("dark");

    act(() => {
      result.current.toggleTheme();
    });
    expect(result.current.theme).toBe("light");
    expect(localStorage.getItem("doc-agent-theme")).toBe("light");
  });

  it("restores stored light theme on mount", () => {
    localStorage.setItem("doc-agent-theme", "light");
    const { result } = renderHook(() => useTheme(), { wrapper: ThemeTestWrapper });
    expect(result.current.theme).toBe("light");
  });
});
