import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ToolChainPanel } from "../components/ToolChainPanel";

describe("ToolChainPanel", () => {
  it("shows empty state", () => {
    render(<ToolChainPanel items={[]} />);
    expect(screen.getByText("工具调用会在这里实时显示。")).toBeInTheDocument();
  });

  it("renders tool cards", () => {
    render(
      <ToolChainPanel
        items={[
          {
            id: "call_1",
            name: "fs_list",
            args: { path: "." },
            status: "done",
          },
        ]}
      />,
    );
    expect(screen.getByText("1. 列出目录")).toBeInTheDocument();
    expect(screen.getByText("完成")).toBeInTheDocument();
    expect(screen.queryByText(/entries/)).not.toBeInTheDocument();
  });

  it("renders streaming progress for pending tool args", () => {
    render(
      <ToolChainPanel
        items={[
          {
            id: "streaming-0",
            name: "skill_run",
            args: undefined,
            status: "streaming",
            argsChars: 12345,
          },
        ]}
      />,
    );
    expect(screen.getByText("生成参数中")).toBeInTheDocument();
    expect(screen.getByText(/已收到 12\.3K 字符/)).toBeInTheDocument();
  });

  it("shows file_busy error on failed tool card", () => {
    render(
      <ToolChainPanel
        items={[
          {
            id: "call_busy",
            name: "fs_write",
            args: { path: "report.docx" },
            status: "error",
            summary: JSON.stringify({
              error: "file_busy",
              path: "report.docx",
              message: "当前 report.docx 已被会话「周报」占用，请稍后重试。",
              blocking_session_id: "sess-1",
            }),
          },
        ]}
      />,
    );
    expect(screen.getByText("失败")).toBeInTheDocument();
    expect(screen.getByText("错误详情")).toBeInTheDocument();
    expect(
      screen.queryByText("当前 report.docx 已被会话「周报」占用，请稍后重试。"),
    ).not.toBeVisible();
  });

  it("scrolls to bottom when a new tool card is appended", () => {
    const scrollIntoView = vi.spyOn(HTMLElement.prototype, "scrollIntoView");

    const first = {
      id: "call_1",
      name: "fs_list",
      args: { path: "." },
      status: "done",
    };
    const second = {
      id: "call_2",
      name: "fs_read",
      args: { path: "a.txt" },
      status: "running",
    };

    const { rerender } = render(<ToolChainPanel items={[first]} />);
    scrollIntoView.mockClear();

    rerender(<ToolChainPanel items={[first, second]} />);
    expect(scrollIntoView).toHaveBeenCalled();

    scrollIntoView.mockRestore();
  });
});
