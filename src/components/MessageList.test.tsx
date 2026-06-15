import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { makeAssistantMessage, makeUserMessage } from "../test/fixtures/messages";
import { MessageList } from "./MessageList";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (command: string) => {
    if (command === "read_attachment_preview") {
      return "data:image/png;base64,abc";
    }
    throw new Error(`unexpected invoke: ${command}`);
  }),
}));

const userMessage = makeUserMessage({ id: "u1", session_id: "s1", content: "hello" });

function assistantMessage(id: string, content: string, reasoning?: string) {
  return makeAssistantMessage({
    id,
    session_id: "s1",
    content,
    reasoning_content: reasoning ?? null,
    seq: Number(id.replace(/\D/g, "")) || 1,
  });
}

describe("MessageList smoke scenarios", () => {
  it("single turn without tools: one persisted assistant, no merged streaming", () => {
    const { rerender } = render(
      <MessageList
        messages={[userMessage]}
        streamingReasoning="thinking"
        streamingContent="partial answer"
        busy
      />,
    );

    expect(screen.getByText("思考中…")).toBeInTheDocument();
    expect(screen.queryByText("思考过程")).not.toBeInTheDocument();

    rerender(
      <MessageList
        messages={[userMessage, assistantMessage("a1", "final answer", "thinking")]}
        streamingReasoning=""
        streamingContent=""
        busy={false}
      />,
    );

    expect(screen.getByText("思考过程")).toBeInTheDocument();
    expect(screen.queryByText("思考中…")).not.toBeInTheDocument();
    expect(screen.getByText("final answer")).toBeInTheDocument();
  });

  it("renders user message attachments when project root is provided", async () => {
    const withImage = makeUserMessage({
      id: "u2",
      session_id: "s1",
      content: "看图",
      attachments_json: JSON.stringify([
        { path: ".cache/attachments/a.png", mime: "image/png" },
      ]),
    });
    render(
      <MessageList
        messages={[withImage]}
        streamingReasoning=""
        streamingContent=""
        busy={false}
        projectId="project-1"
      />,
    );
    expect(await screen.findByRole("button", { name: /消息图片附件/ })).toBeInTheDocument();
  });

  it("renders answered clarify cards from tool call records", () => {
    const assistant = assistantMessage("a1", "", "ask clarify");
    render(
      <MessageList
        messages={[userMessage, assistant]}
        toolCalls={[
          {
            id: "call1",
            message_id: "a1",
            name: "clarify_ask",
            args_json: JSON.stringify({
              id: "style",
              kind: "single",
              prompt: "选择视觉风格",
              options: [{ id: "business", label: "商务深色" }],
            }),
            result_json: JSON.stringify({
              question_id: "style",
              selected: ["business"],
              custom: null,
              display_text: "商务深色",
            }),
            status: "done",
            duration_ms: 0,
            created_at: "now",
          },
        ]}
        streamingReasoning=""
        streamingContent=""
        busy={false}
      />,
    );

    expect(screen.getByText("选择视觉风格")).toBeInTheDocument();
    expect(screen.getByText("已回答：商务深色")).toBeInTheDocument();
  });

  it("single tool one step: two assistant bubbles, streaming resets between steps", () => {
    const step1 = assistantMessage("a1", "", "plan tool");
    const { rerender } = render(
      <MessageList
        messages={[userMessage, step1]}
        streamingReasoning="step2 think"
        streamingContent="step2 answer"
        activity="正在执行「列出目录」…"
        busy
      />,
    );

    expect(screen.getAllByText("思考过程")).toHaveLength(1);
    expect(screen.getByText("思考中…")).toBeInTheDocument();
    expect(screen.getByText(/正在执行/)).toBeInTheDocument();

    rerender(
      <MessageList
        messages={[userMessage, step1, assistantMessage("a2", "done", "step2 think")]}
        streamingReasoning=""
        streamingContent=""
        busy={false}
      />,
    );

    expect(screen.getAllByText("思考过程")).toHaveLength(2);
    expect(screen.queryByText("思考中…")).not.toBeInTheDocument();
  });

  it("multi tool three steps: three persisted assistants, never more than one streaming bubble", () => {
    const midMessages = [
      userMessage,
      assistantMessage("a1", "", "step1"),
      assistantMessage("a2", "", "step2"),
    ];
    const finalMessages = [...midMessages, assistantMessage("a3", "final", "step3")];
    const { rerender } = render(
      <MessageList
        messages={midMessages}
        streamingReasoning="step3"
        streamingContent="final"
        activity="正在执行工具…"
        busy
      />,
    );

    expect(screen.getAllByText("思考过程")).toHaveLength(2);
    expect(screen.getByText("思考中…")).toBeInTheDocument();

    rerender(
      <MessageList
        messages={finalMessages}
        streamingReasoning=""
        streamingContent=""
        busy={false}
      />,
    );

    expect(screen.getAllByText("思考过程")).toHaveLength(3);
    expect(screen.queryByText("思考中…")).not.toBeInTheDocument();
  });
});
