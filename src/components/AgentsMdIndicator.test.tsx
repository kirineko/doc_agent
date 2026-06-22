import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { AgentsMdIndicator } from "./AgentsMdIndicator";

describe("AgentsMdIndicator", () => {
  it("hides when idle", () => {
    const { container } = render(<AgentsMdIndicator status="idle" />);
    expect(container).toBeEmptyDOMElement();
  });

  it("shows loaded label", () => {
    render(<AgentsMdIndicator status="loaded" variant="labeled" />);
    expect(screen.getByText("已加载 AGENTS.md")).toBeInTheDocument();
  });

  it("shows missing label with hint", () => {
    render(<AgentsMdIndicator status="missing" variant="labeled" />);
    const label = screen.getByText("未配置 AGENTS.md");
    expect(label.closest("[title]")).toHaveAttribute("title", expect.stringContaining("/init"));
  });
});
