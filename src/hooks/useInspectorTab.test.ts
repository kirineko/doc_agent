import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { LiveToolCall } from "../components/ToolChainPanel";
import { useInspectorTab } from "./useInspectorTab";

const activeTools: LiveToolCall[] = [
  {
    id: "tool-1",
    name: "read_file",
    args: {},
    status: "running",
  },
];

describe("useInspectorTab", () => {
  it("clears manual pin when active session changes", () => {
    const { result, rerender } = renderHook(
      ({
        activeSessionId,
        liveTools,
        turnNonce,
      }: {
        activeSessionId?: string;
        liveTools: LiveToolCall[];
        turnNonce: number;
      }) =>
        useInspectorTab({
          liveTools,
          turnNonce,
          activeSessionId,
        }),
      {
        initialProps: {
          activeSessionId: "session-a",
          liveTools: [] as LiveToolCall[],
          turnNonce: 0,
        },
      },
    );

    act(() => {
      result.current.setTab("files", true);
    });

    rerender({
      activeSessionId: "session-a",
      liveTools: activeTools,
      turnNonce: 0,
    });
    expect(result.current.tab).toBe("files");

    rerender({
      activeSessionId: "session-b",
      liveTools: activeTools,
      turnNonce: 0,
    });
    expect(result.current.tab).toBe("toolchain");
  });
});
