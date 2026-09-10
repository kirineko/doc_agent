import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { retryNoticeText, TurnErrorCard } from "./TurnErrorCard";

describe("TurnErrorCard", () => {
  it("shows auth headline and hint, expanding detail", async () => {
    render(
      <TurnErrorCard
        error={{
          message: "模型服务鉴权失败：Invalid API key",
          code: "auth",
          hint: "请在「密钥与服务」中检查该模型的 API Key",
          detail: '{"error":"invalid_api_key"}',
        }}
      />,
    );
    expect(screen.getByText("模型服务鉴权失败：Invalid API key")).toBeVisible();
    expect(screen.getByText("请在「密钥与服务」中检查该模型的 API Key")).toBeVisible();
    expect(screen.queryByText(/invalid_api_key/)).not.toBeVisible();
    await userEvent.click(screen.getByText("详情"));
    expect(screen.getByText(/invalid_api_key/)).toBeVisible();
  });

  it("copies detail to the clipboard", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, { clipboard: { writeText } });
    render(
      <TurnErrorCard
        error={{
          message: "模型服务鉴权失败：Invalid API key",
          detail: '{"error":"invalid_api_key"}',
        }}
      />,
    );
    await userEvent.click(screen.getByText("详情"));
    await userEvent.click(screen.getByText("复制详情"));
    expect(writeText).toHaveBeenCalledWith('{"error":"invalid_api_key"}');
  });

  it("formats retry notice text", () => {
    expect(retryNoticeText({ attempt: 1, max: 2, kind: "network", delayMs: 1000 })).toBe(
      "网络波动，正在重试（1/2）…",
    );
  });
});
