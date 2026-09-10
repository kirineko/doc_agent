import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { MODEL_OPTIONS } from "../types";
import { ModelConfigSection } from "./ModelConfigSection";

describe("ModelConfigSection", () => {
  it("preserves a legacy Flash selection and allows enabling thinking", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<ModelConfigSection config={{ model: "deepseek-v4-flash", thinking_enabled: false, thinking_effort: "max" }} locked={false} onChange={onChange} />);
    expect(screen.getByRole("combobox")).toHaveValue("deepseek-flash");
    expect(screen.getByRole("checkbox", { name: "启用思考模式" })).not.toBeChecked();
    expect(screen.queryByText("此模型始终启用思考")).not.toBeInTheDocument();
    expect(onChange).not.toHaveBeenCalled();
    await user.click(screen.getByRole("checkbox", { name: "启用思考模式" }));
    expect(onChange).toHaveBeenCalledWith({ thinking_enabled: true });
  });

  it("shows the original effort for a locked legacy Flash session", () => {
    render(<ModelConfigSection config={{ model: "deepseek-v4-flash", thinking_enabled: true, thinking_effort: "max" }} locked onChange={() => undefined} />);
    expect(screen.getByText("DeepSeek Flash · max")).toBeInTheDocument();
    expect(screen.queryByRole("combobox")).not.toBeInTheDocument();
  });

  it("shows six selectable models and gemini medium efforts", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(
      <ModelConfigSection
        config={{ model: "gemini-3.8-flash", thinking_enabled: true, thinking_effort: "medium" }}
        locked={false}
        models={MODEL_OPTIONS}
        onChange={onChange}
      />,
    );
    expect(screen.getByText("此模型始终启用思考")).toBeInTheDocument();
    expect(screen.queryByLabelText("启用思考模式")).not.toBeInTheDocument();
    expect(screen.getByRole("radio", { name: "medium" })).toBeChecked();
    expect(screen.getByRole("radio", { name: "low" })).toBeInTheDocument();
    expect(screen.getByRole("radio", { name: "high" })).toBeInTheDocument();
    expect(screen.queryByRole("radio", { name: "max" })).not.toBeInTheDocument();
    expect(screen.queryByRole("option", { name: "Kimi K2.6" })).not.toBeInTheDocument();
    await user.click(screen.getByRole("radio", { name: "low" }));
    expect(onChange).toHaveBeenCalledWith({ thinking_effort: "low" });
  });

  it("shows only a thinking toggle for MiMo", () => {
    render(
      <ModelConfigSection
        config={{ model: "mimo-v2.5", thinking_enabled: true, thinking_effort: "high" }}
        locked={false}
        models={MODEL_OPTIONS}
        onChange={() => undefined}
      />,
    );
    expect(screen.getByLabelText("启用思考模式")).toBeInTheDocument();
    expect(screen.getAllByRole("combobox")).toHaveLength(1);
  });

  it("locks historical ultraspeed with a retired notice", () => {
    render(
      <ModelConfigSection
        config={{
          model: "mimo-v2.5-pro-ultraspeed",
          thinking_enabled: true,
          thinking_effort: "high",
        }}
        locked
        models={MODEL_OPTIONS}
        onChange={() => undefined}
      />,
    );
    expect(screen.getAllByText(/已不可调用/).length).toBeGreaterThan(0);
    expect(screen.queryByRole("combobox")).not.toBeInTheDocument();
  });
});
