import { describe, expect, it } from "vitest";
import { activeClarifyFromBundle, normalizeBriefEntries } from "./clarifyBrief";
import type { MessageBundle } from "../types";

describe("normalizeBriefEntries", () => {
  it("unwraps single nested wrapper object", () => {
    const entries = normalizeBriefEntries({
      创作简报: {
        文档类型: "PPT",
        "主题/目标": "年度总结",
      },
    });
    expect(entries).toEqual([
      ["文档类型", "PPT"],
      ["主题/目标", "年度总结"],
    ]);
  });

  it("keeps flat brief as-is", () => {
    const entries = normalizeBriefEntries({
      文档类型: "Word",
      排版风格: "**简约留白**",
    });
    expect(entries).toEqual([
      ["文档类型", "Word"],
      ["排版风格", "**简约留白**"],
    ]);
  });

  it("flattens nested brief field values", () => {
    const entries = normalizeBriefEntries({
      创作简报: {
        文档类型: "PPT",
        样式要点: { 字体: "微软雅黑", 配色: "科技蓝" },
      },
    });
    expect(entries).toEqual([
      ["文档类型", "PPT"],
      ["样式要点", "字体：微软雅黑\n配色：科技蓝"],
    ]);
  });
});

describe("activeClarifyFromBundle", () => {
  const question = {
    id: "q1",
    kind: "single",
    prompt: "选择文档类型",
    options: [{ id: "word", label: "Word" }],
  };

  it("returns the pending clarify card when clarify_pending exists", () => {
    const bundle: MessageBundle = {
      messages: [],
      tool_calls: [
        {
          id: "call_1",
          message_id: "m1",
          name: "clarify_ask",
          args_json: JSON.stringify(question),
          status: "awaiting_user",
          duration_ms: 0,
          created_at: "now",
        },
      ],
      clarify_pending: {
        session_id: "s1",
        turn_id: "t1",
        tool_call_id: "call_1",
        question_json: JSON.stringify(question),
        created_at: "now",
      },
    };
    expect(activeClarifyFromBundle(bundle)).toEqual({
      toolCallId: "call_1",
      question,
    });
  });

  it("returns undefined once the clarify is consumed (resume failed after answer persisted)", () => {
    // 复现假死修复点：澄清答案已写入 tool result（status=done）、pending 被删除，
    // 即使后续 LLM 调用失败，也不应再展示澄清卡片，否则会冻结输入框。
    const bundle: MessageBundle = {
      messages: [],
      tool_calls: [
        {
          id: "call_1",
          message_id: "m1",
          name: "clarify_ask",
          args_json: JSON.stringify(question),
          result_json: JSON.stringify({ question_id: "q1", selected: ["word"] }),
          status: "done",
          duration_ms: 0,
          created_at: "now",
        },
      ],
      clarify_pending: null,
    };
    expect(activeClarifyFromBundle(bundle)).toBeUndefined();
  });
});
