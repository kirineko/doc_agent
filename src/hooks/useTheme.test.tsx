import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import { ThemeTestWrapper } from "../test/renderWithTheme";
import { useTheme } from "./useTheme";

describe("useTheme", () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.removeAttribute("data-theme");
  });

  it("defaults to dark theme", () => {
    const { result } = renderHook(() => useTheme(), { wrapper: ThemeTestWrapper });
    expect(result.current.theme).toBe("dark");
  });

  it("toggleTheme switches dark and light", () => {
    const { result } = renderHook(() => useTheme(), { wrapper: ThemeTestWrapper });

    act(() => {
      result.current.toggleTheme();
    });
    expect(result.current.theme).toBe("light");
    expect(localStorage.getItem("doc-agent-theme")).toBe("light");

    act(() => {
      result.current.toggleTheme();
    });
    expect(result.current.theme).toBe("dark");
    expect(localStorage.getItem("doc-agent-theme")).toBe("dark");
  });

  it("restores stored light theme on mount", () => {
    localStorage.setItem("doc-agent-theme", "light");
    const { result } = renderHook(() => useTheme(), { wrapper: ThemeTestWrapper });
    expect(result.current.theme).toBe("light");
  });
});
