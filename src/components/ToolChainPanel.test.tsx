import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
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
});
