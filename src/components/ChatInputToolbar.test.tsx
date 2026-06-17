import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { ChatInputToolbar } from "./ChatInputToolbar";

const baseProps = {
  disabled: false,
  projectSelected: true,
  supportsVision: false,
  slashMenuOpen: false,
  canSend: true,
  busy: true,
  onSend: vi.fn(),
  onImportFiles: vi.fn(),
  onPickImage: vi.fn(),
  onToggleSlashMenu: vi.fn(),
};

describe("ChatInputToolbar", () => {
  it("shows Stop button when showStop is true", async () => {
    const onStop = vi.fn();
    render(
      <ChatInputToolbar
        {...baseProps}
        disabled
        showStop
        onStop={onStop}
      />,
    );

    const stopButton = screen.getByRole("button", { name: "停止" });
    expect(stopButton).toBeEnabled();
    await userEvent.click(stopButton);
    expect(onStop).toHaveBeenCalledOnce();
  });

  it("hides Send button when Stop is shown", () => {
    render(
      <ChatInputToolbar
        {...baseProps}
        showStop
        onStop={vi.fn()}
      />,
    );

    expect(screen.queryByRole("button", { name: "发送" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "发送中" })).not.toBeInTheDocument();
  });

  it("disables Stop while stopping", () => {
    render(
      <ChatInputToolbar
        {...baseProps}
        showStop
        stopping
        onStop={vi.fn()}
      />,
    );

    expect(screen.getByRole("button", { name: "停止中…" })).toBeDisabled();
  });

  it("disables Send when parallel capacity is reached", () => {
    render(
      <ChatInputToolbar
        {...baseProps}
        busy={false}
        sendBlockedByParallel
      />,
    );

    expect(screen.getByRole("button", { name: "发送" })).toBeDisabled();
  });
});
