import { render, screen, fireEvent } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { BuildArtifactsPanel } from "./BuildArtifactsPanel";
import type { TurnArtifact } from "../lib/agentEvents";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const artifacts: TurnArtifact[] = [
  { path: "out/report.docx", sourceToolCallId: "c1", sourceToolLabel: "运行 Skill" },
  { path: "out/data.xlsx", sourceToolCallId: "c2", sourceToolLabel: "写入 Excel" },
  { path: "docs/notes.md", sourceToolCallId: "c3", sourceToolLabel: "写入文件" },
];

describe("BuildArtifactsPanel", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("shows empty state when no artifacts", () => {
    render(<BuildArtifactsPanel artifacts={[]} projectId="p1" />);
    expect(screen.getByText("本轮没有产生或修改文件。")).toBeInTheDocument();
  });

  it("renders all artifact paths and source tool labels", () => {
    render(<BuildArtifactsPanel artifacts={artifacts} projectId="p1" />);
    // 路径与 baseName 可能重合（单段路径），用 title 精确定位完整路径行
    expect(screen.getByText("report.docx")).toBeInTheDocument();
    expect(screen.getByText("data.xlsx")).toBeInTheDocument();
    expect(screen.getByText("notes.md")).toBeInTheDocument();
    expect(screen.getByText(/运行 Skill/)).toBeInTheDocument();
  });

  it("invokes open_project_file when clicking 打开", () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    render(<BuildArtifactsPanel artifacts={artifacts} projectId="p1" />);

    const openButtons = screen.getAllByTitle("用默认程序打开");
    fireEvent.click(openButtons[0]);

    expect(invoke).toHaveBeenCalledWith("open_project_file", {
      projectId: "p1",
      relativePath: "out/report.docx",
    });
  });

  it("invokes reveal_project_file when clicking 定位", () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    render(<BuildArtifactsPanel artifacts={artifacts} projectId="p1" />);

    fireEvent.click(screen.getAllByTitle("在文件夹中显示")[0]);

    expect(invoke).toHaveBeenCalledWith("reveal_project_file", {
      projectId: "p1",
      relativePath: "out/report.docx",
    });
  });

  it("shows inline error when open fails", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("not found"));
    render(<BuildArtifactsPanel artifacts={artifacts} projectId="p1" />);

    fireEvent.click(screen.getAllByTitle("用默认程序打开")[0]);

    expect(await screen.findByText(/not found/)).toBeInTheDocument();
  });

  it("shows 打开 for directory artifacts (open/reveal uniform, no slash)", () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    // 目录路径不再携带尾部斜杠约定；打开/定位对文件与目录统一可用
    const dirArtifact: TurnArtifact = {
      path: "output",
      sourceToolCallId: "c1",
      sourceToolLabel: "拆分 PDF",
    };
    render(<BuildArtifactsPanel artifacts={[dirArtifact]} projectId="p1" />);

    fireEvent.click(screen.getByTitle("用默认程序打开"));

    expect(invoke).toHaveBeenCalledWith("open_project_file", {
      projectId: "p1",
      relativePath: "output",
    });
  });
});
