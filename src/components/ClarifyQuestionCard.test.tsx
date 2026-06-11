import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { ClarifyQuestionCard } from "./ClarifyQuestionCard";
import type { ClarifyQuestion } from "../types";

const singleQuestion: ClarifyQuestion = {
  id: "style",
  kind: "single",
  prompt: "选择视觉风格",
  options: [
    { id: "minimal", label: "简约留白" },
    { id: "business", label: "商务深色" },
  ],
  allow_custom: true,
};

describe("ClarifyQuestionCard", () => {
  it("submits a selected single option", async () => {
    const onSubmit = vi.fn();
    render(<ClarifyQuestionCard question={singleQuestion} onSubmit={onSubmit} />);

    await userEvent.click(screen.getByText("商务深色"));
    await userEvent.click(screen.getByText("提交回答"));

    expect(onSubmit).toHaveBeenCalledWith({ selected: ["business"], custom: null });
  });

  it("keeps multi submit disabled until min selections are met", async () => {
    const onSubmit = vi.fn();
    const question: ClarifyQuestion = {
      id: "sections",
      kind: "multi",
      prompt: "选择章节",
      min_selections: 2,
      options: [
        { id: "summary", label: "摘要" },
        { id: "analysis", label: "分析" },
        { id: "actions", label: "行动项" },
      ],
    };
    render(<ClarifyQuestionCard question={question} onSubmit={onSubmit} />);

    expect(screen.getByText("提交回答")).toBeDisabled();
    await userEvent.click(screen.getByText("摘要"));
    expect(screen.getByText("提交回答")).toBeDisabled();
    await userEvent.click(screen.getByText("分析"));
    await userEvent.click(screen.getByText("提交回答"));

    expect(onSubmit).toHaveBeenCalledWith({
      selected: ["summary", "analysis"],
      custom: null,
    });
  });

  it("submits text answer via custom input", async () => {
    const onSubmit = vi.fn();
    const question: ClarifyQuestion = {
      id: "topic",
      kind: "text",
      prompt: "这份文档的主题是什么？",
    };
    render(<ClarifyQuestionCard question={question} onSubmit={onSubmit} />);

    expect(screen.getByText("提交回答")).toBeDisabled();
    await userEvent.type(screen.getByPlaceholderText("其他 / 自定义回答"), "年度销售总结");
    await userEvent.click(screen.getByText("提交回答"));

    expect(onSubmit).toHaveBeenCalledWith({ selected: [], custom: "年度销售总结" });
  });

  it("renders brief and submits confirm for confirm_brief", async () => {
    const onSubmit = vi.fn();
    const question: ClarifyQuestion = {
      id: "brief",
      kind: "confirm_brief",
      prompt: "请确认创作简报",
      brief: { 文档类型: "PPT", 主题: "年度总结" },
    };
    render(<ClarifyQuestionCard question={question} onSubmit={onSubmit} />);

    expect(screen.getByText("文档类型")).toBeInTheDocument();
    expect(screen.getByText("PPT")).toBeInTheDocument();
    expect(screen.getByText("提交回答")).toBeDisabled();
    await userEvent.click(screen.getByText("确认继续"));
    await userEvent.click(screen.getByText("提交回答"));

    expect(onSubmit).toHaveBeenCalledWith({ selected: ["confirm"], custom: null });
  });

  it("clears confirm selection when modification feedback is typed", async () => {
    const onSubmit = vi.fn();
    const question: ClarifyQuestion = {
      id: "brief",
      kind: "confirm_brief",
      prompt: "请确认创作简报",
      brief: { 主题: "年度总结" },
    };
    render(<ClarifyQuestionCard question={question} onSubmit={onSubmit} />);

    await userEvent.click(screen.getByText("确认继续"));
    await userEvent.type(
      screen.getByPlaceholderText("如需修改，请写下修改意见"),
      "把主题改成季度复盘",
    );
    await userEvent.click(screen.getByText("提交回答"));

    expect(onSubmit).toHaveBeenCalledWith({
      selected: [],
      custom: "把主题改成季度复盘",
    });
  });

  it("renders nested brief entries instead of [object Object]", () => {
    const question: ClarifyQuestion = {
      id: "brief",
      kind: "confirm_brief",
      prompt: "请确认创作简报",
      brief: {
        创作简报: {
          文档类型: "PPT",
          排版风格: "**简约留白**",
        },
      } as unknown as ClarifyQuestion["brief"],
    };
    render(<ClarifyQuestionCard question={question} onSubmit={vi.fn()} />);

    expect(screen.getByText("文档类型")).toBeInTheDocument();
    expect(screen.getByText("PPT")).toBeInTheDocument();
    expect(screen.queryByText("[object Object]")).not.toBeInTheDocument();
    expect(document.querySelector("strong")?.textContent).toBe("简约留白");
  });

  it("renders markdown bold in prompt", () => {
    const question: ClarifyQuestion = {
      id: "brief",
      kind: "confirm_brief",
      prompt: "请确认**创作简报**",
      brief: { 文档类型: "PPT" },
    };
    render(<ClarifyQuestionCard question={question} onSubmit={vi.fn()} />);

    expect(document.querySelector("strong")?.textContent).toBe("创作简报");
    expect(screen.queryByText("**创作简报**")).not.toBeInTheDocument();
  });

  it("renders readonly answered state", () => {
    render(
      <ClarifyQuestionCard
        question={singleQuestion}
        answer={{
          question_id: "style",
          selected: ["business"],
          custom: null,
          display_text: "商务深色",
        }}
      />,
    );

    expect(screen.getByText("已回答：商务深色")).toBeInTheDocument();
    expect(screen.queryByText("提交回答")).not.toBeInTheDocument();
  });
});
