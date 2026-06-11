import { parseJson } from "./json";
import type {
  ClarifyAnswer,
  ClarifyQuestion,
  Message,
  MessageBundle,
  ToolCallRecord,
} from "../types";

export interface ActiveClarify {
  toolCallId: string;
  question: ClarifyQuestion;
}

/** 将 confirm_brief 的 brief 规范为可展示的 [字段, 文本] 列表 */
export function normalizeBriefEntries(
  brief: Record<string, unknown> | null | undefined,
): Array<[string, string]> {
  if (!brief) return [];

  let entries = Object.entries(brief);
  if (entries.length === 1) {
    const [key, value] = entries[0]!;
    if (isBriefWrapperKey(key) && value && typeof value === "object" && !Array.isArray(value)) {
      entries = Object.entries(value as Record<string, unknown>);
    }
  }

  return entries
    .map(([key, value]) => [key, briefValueToText(value)] as [string, string])
    .filter(([, value]) => value.length > 0);
}

export function parseClarifyFromJson(text: string | null | undefined): ClarifyQuestion | null {
  return parseClarifyQuestion(parseJson(text));
}

/** 从 tool args / pending JSON 解析并规范化 ClarifyQuestion（兼容历史 raw args） */
export function parseClarifyQuestion(raw: unknown): ClarifyQuestion | null {
  if (!raw || typeof raw !== "object") return null;
  const question = raw as ClarifyQuestion;
  if (!question.id || !question.kind || !question.prompt) return null;
  if (!question.brief) return question;

  const entries = normalizeBriefEntries(question.brief as Record<string, unknown>);
  if (entries.length === 0) return question;

  return {
    ...question,
    brief: Object.fromEntries(entries),
  };
}

export function activeClarifyFromBundle(bundle: MessageBundle): ActiveClarify | undefined {
  if (bundle.clarify_pending) {
    const question = parseClarifyFromJson(bundle.clarify_pending.question_json);
    if (question) {
      return { toolCallId: bundle.clarify_pending.tool_call_id, question };
    }
  }
  const pending = bundle.tool_calls.find(
    (call) => call.name === "clarify_ask" && call.status === "awaiting_user",
  );
  if (!pending) return undefined;
  const question = parseClarifyFromJson(pending.args_json);
  if (!question) return undefined;
  return { toolCallId: pending.id, question };
}

export function messageBundleState(bundle: MessageBundle): {
  messages: Message[];
  toolCalls: ToolCallRecord[];
  activeClarify?: ActiveClarify;
} {
  return {
    messages: bundle.messages,
    toolCalls: bundle.tool_calls,
    activeClarify: activeClarifyFromBundle(bundle),
  };
}

export function parseAnsweredClarifyCall(
  call: ToolCallRecord,
): { callId: string; question: ClarifyQuestion; answer: ClarifyAnswer } | null {
  if (call.name !== "clarify_ask" || call.status === "awaiting_user") return null;
  const question = parseClarifyFromJson(call.args_json);
  const answer = parseJson<ClarifyAnswer>(call.result_json);
  if (!question || !answer) return null;
  return { callId: call.id, question, answer };
}

function briefValueToText(value: unknown): string {
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  if (value == null) return "";
  if (Array.isArray(value)) {
    return value
      .map((item) => briefValueToText(item))
      .filter(Boolean)
      .join("、");
  }
  if (typeof value === "object") {
    const lines = Object.entries(value as Record<string, unknown>)
      .map(([field, nested]) => {
        const text = briefValueToText(nested);
        return text ? `${field}：${text}` : "";
      })
      .filter(Boolean);
    if (lines.length > 0) return lines.join("\n");
    try {
      return JSON.stringify(value, null, 2);
    } catch {
      return String(value);
    }
  }
  return String(value);
}

function isBriefWrapperKey(key: string): boolean {
  const trimmed = key.trim();
  const lower = trimmed.toLowerCase();
  return [
    "创作简报",
    "【创作简报】",
    "brief",
    "Brief",
    "summary",
    "Summary",
    "创作简报摘要",
    "creation_brief",
  ].some((wrapper) => wrapper.toLowerCase() === lower || wrapper === trimmed);
}
