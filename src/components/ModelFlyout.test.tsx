import { useRef, type ComponentProps } from "react";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MODEL_OPTIONS } from "../types";
import { applyTheme } from "../lib/theme";
import { ModelFlyout } from "./ModelFlyout";

const geminiConfig = {
  model: "gemini-3.8-flash",
  thinking_enabled: true,
  thinking_effort: "medium",
};

const keysReady = {
  deepseek: true,
  kimi: true,
  mimo: true,
  google: true,
  zhipu: true,
};

function FlyoutHarness(props: Partial<ComponentProps<typeof ModelFlyout>>) {
  const triggerRef = useRef<HTMLButtonElement>(null);
  return (
    <>
      <button ref={triggerRef} type="button">
        trigger
      </button>
      <ModelFlyout
        open
        triggerRef={triggerRef}
        models={MODEL_OPTIONS}
        config={geminiConfig}
        locked={false}
        apiKeyStatus={keysReady}
        onClose={() => undefined}
        onChange={props.onChange ?? (() => undefined)}
        {...props}
      />
    </>
  );
}

describe("ModelFlyout", () => {
  it("recognizes a legacy Flash session without resetting its thinking configuration", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<FlyoutHarness config={{ model: "deepseek-v4-flash", thinking_enabled: true, thinking_effort: "max" }} onChange={onChange} />);
    await screen.findByRole("dialog");
    expect(screen.getByRole("radio", { name: "DeepSeek" })).toBeChecked();
    const model = screen.getByRole("radio", { name: /DeepSeek Flash/ });
    expect(model).toBeChecked();
    expect(screen.getByRole("checkbox", { name: "启用思考模式" })).toBeChecked();
    expect(screen.getByRole("radio", { name: "max" })).toBeChecked();
    await user.click(model);
    await user.click(screen.getByRole("radio", { name: "DeepSeek" }));
    expect(onChange).not.toHaveBeenCalled();
    await user.click(screen.getByRole("radio", { name: "low" }));
    expect(onChange).toHaveBeenCalledWith({ thinking_effort: "low" });
  });

  it("shows disabled thinking for a locked legacy Flash session", async () => {
    render(<FlyoutHarness locked config={{ model: "deepseek-v4-flash", thinking_enabled: false, thinking_effort: "max" }} />);
    await screen.findByRole("dialog");
    expect(screen.getByText("DeepSeek Flash · 思考关闭")).toBeInTheDocument();
    expect(screen.queryByRole("checkbox")).not.toBeInTheDocument();
  });

  beforeEach(() => {
    vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockReturnValue({
      x: 40,
      y: 400,
      top: 400,
      bottom: 428,
      left: 40,
      right: 240,
      width: 200,
      height: 28,
      toJSON: () => ({}),
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
    document.documentElement.removeAttribute("data-theme");
  });

  it("shows wrapped providers and themed effort chips instead of a native select", async () => {
    render(<FlyoutHarness />);
    const dialog = await screen.findByRole("dialog", { name: "模型" });
    expect(dialog).toBeInTheDocument();
    expect(screen.getByRole("radio", { name: "DeepSeek" })).toBeInTheDocument();
    expect(screen.getByRole("radio", { name: "MiMo" })).toBeInTheDocument();
    expect(screen.getByRole("radio", { name: "Kimi" })).toBeInTheDocument();
    expect(screen.getByRole("radio", { name: "Google Gemini" })).toBeChecked();
    expect(screen.getByRole("radio", { name: "智谱 GLM" })).toBeInTheDocument();
    expect(screen.queryByRole("combobox")).not.toBeInTheDocument();
    expect(screen.getByRole("radio", { name: "medium" })).toHaveClass("chip-surface-selected");
    expect(screen.getByRole("radio", { name: "low" })).toHaveClass("chip-surface");
  });

  it("keeps chip theme classes in light and dark", async () => {
    for (const theme of ["light", "dark"] as const) {
      applyTheme(theme);
      const { unmount } = render(<FlyoutHarness />);
      const medium = await screen.findByRole("radio", { name: "medium" });
      expect(document.documentElement.dataset.theme).toBe(theme);
      expect(medium).toHaveClass("chip-surface");
      expect(medium).toHaveClass("chip-surface-selected");
      expect(screen.getByRole("radio", { name: "Google Gemini" })).toHaveClass("chip-surface");
      expect(screen.queryByRole("combobox")).not.toBeInTheDocument();
      unmount();
    }
  });

  it("updates thinking effort from the chip group", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<FlyoutHarness onChange={onChange} />);
    await screen.findByRole("dialog");
    await user.click(screen.getByRole("radio", { name: "low" }));
    expect(onChange).toHaveBeenCalledWith({ thinking_effort: "low" });
  });

  it("marks providers that are missing an API key", async () => {
    render(<FlyoutHarness apiKeyStatus={{ ...keysReady, google: false }} />);
    await screen.findByRole("dialog");
    expect(screen.getByRole("radio", { name: /Google Gemini/ })).toHaveAccessibleName(
      "Google Gemini · 未配置 API Key",
    );
  });

  it("does not expose a native select while locked", async () => {
    render(<FlyoutHarness locked />);
    await screen.findByRole("dialog");
    expect(screen.getByText("当前会话已锁定模型配置。")).toBeInTheDocument();
    expect(screen.queryByRole("combobox")).not.toBeInTheDocument();
    expect(screen.queryByRole("radio", { name: "medium" })).not.toBeInTheDocument();
  });

  it("waits for the trigger geometry before painting", async () => {
    render(<FlyoutHarness open={false} />);
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    await waitFor(() => {
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });
  });
});
