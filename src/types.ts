export interface ProjectFileEntry {
  path: string;
  is_dir: boolean;
  modified_ms: number;
}

export interface ProjectFileList {
  entries: ProjectFileEntry[];
  truncated: boolean;
}

export interface ProjectDirEntry {
  name: string;
  is_dir: boolean;
}

export interface ProjectDirListing {
  path: string;
  entries: ProjectDirEntry[];
}

export interface Project {
  id: string;
  name: string;
  root_path: string;
  created_at: string;
}

export interface Session {
  id: string;
  project_id: string;
  title: string;
  model: string;
  thinking_enabled: boolean;
  thinking_effort: string;
  created_at: string;
  updated_at: string;
}


export interface ClarifyOption {
  id: string;
  label: string;
  hint?: string | null;
}

export type ClarifyKind = "single" | "multi" | "text" | "confirm_brief" | "confirm_agents_md";

export interface ClarifyQuestion {
  id: string;
  kind: ClarifyKind;
  prompt: string;
  description?: string | null;
  options?: ClarifyOption[];
  allow_custom?: boolean;
  custom_label?: string | null;
  custom_placeholder?: string | null;
  min_selections?: number | null;
  max_selections?: number | null;
  brief?: Record<string, string> | null;
  preview_markdown?: string | null;
  changelog_summary?: string | null;
}

export interface ClarifyAnswer {
  question_id: string;
  selected: string[];
  custom?: string | null;
  display_text: string;
  brief?: Record<string, string> | null;
  preview_markdown?: string | null;
}

export interface SubmitClarifyAnswerRequest {
  session_id: string;
  question_id: string;
  selected?: string[];
  custom?: string | null;
}

export interface MessageAttachment {
  path: string;
  mime: string;
}

export interface Message {
  id: string;
  session_id: string;
  role: string;
  content?: string | null;
  reasoning_content?: string | null;
  tool_call_id?: string | null;
  seq: number;
  created_at: string;
  attachments_json?: string | null;
}

export type ModelAvailability = "available" | "retired";

export interface ModelInfo {
  id: string;
  label: string;
  provider: string;
  api_model: string;
  supports_vision: boolean;
  supports_effort: boolean;
  selectable: boolean;
  availability: ModelAvailability;
  supports_thinking_toggle: boolean;
  thinking_efforts: string[];
  default_thinking_enabled: boolean;
  default_thinking_effort: string;
  max_context: number;
  max_output_tokens?: number | null;
}

export interface ToolCallRecord {
  id: string;
  message_id: string;
  name: string;
  args_json: string;
  result_json?: string | null;
  status: string;
  duration_ms: number;
  created_at: string;
}

export interface MessageBundle {
  messages: Message[];
  tool_calls: ToolCallRecord[];
  clarify_pending?: ClarifyPending | null;
}

export interface ClarifyPending {
  session_id: string;
  turn_id: string;
  tool_call_id: string;
  question_json: string;
  created_at: string;
}

export interface CompactSessionResponse {
  compacted: boolean;
  before_tokens: number;
  after_tokens: number;
  reason?: string | null;
}

export type AgentEvent =
  | { kind: "reasoning_token"; session_id: string; turn_id: string; delta: string }
  | { kind: "content_token"; session_id: string; turn_id: string; delta: string }
  | { kind: "tool_call_stream"; session_id: string; turn_id: string; index: number; name: string; args_chars: number }
  | { kind: "tool_call"; session_id: string; turn_id: string; id: string; name: string; args: unknown; status: string; index?: number }
  | {
      kind: "tool_result";
      session_id: string;
      turn_id: string;
      id: string;
      ok: boolean;
      summary: string;
      duration_ms: number;
      changed_paths?: string[];
    }
  | { kind: "turn_complete"; session_id: string; turn_id: string }
  | { kind: "turn_cancelled"; session_id: string; turn_id: string }
  | { kind: "turn_awaiting_user"; session_id: string; turn_id: string }
  | { kind: "clarify_question"; session_id: string; turn_id: string; tool_call_id: string; question: ClarifyQuestion }
  | {
      kind: "assistant_step_done";
      session_id: string;
      turn_id: string;
      message: Message;
    }
  | {
      kind: "context_usage";
      session_id: string;
      used_tokens: number;
      max_tokens: number;
      ratio: number;
    }
  | {
      kind: "context_compacted";
      session_id: string;
      before_tokens: number;
      after_tokens: number;
      trigger: "auto" | "manual";
    }
  | {
      kind: "compaction_started";
      session_id: string;
      trigger: "auto" | "manual";
    }
  | {
      kind: "error";
      session_id: string;
      turn_id: string;
      message: string;
      code?: FailureKind;
      retryable?: boolean;
      detail?: string;
      hint?: string;
    }
  | {
      kind: "provider_retry";
      session_id: string;
      turn_id: string;
      attempt: number;
      max: number;
      code: FailureKind;
      delay_ms: number;
    }
  | { kind: "session_title_updated"; session_id: string; title: string };

export type FailureKind =
  | "auth"
  | "rate_limit"
  | "bad_request"
  | "payload_too_large"
  | "context_length"
  | "server"
  | "network"
  | "timeout"
  | "stream_error"
  | "stream_incomplete";

/** `error` 事件去掉路由字段后的载荷，UI 直接渲染 */
export type TurnError = Omit<
  Extract<AgentEvent, { kind: "error" }>,
  "kind" | "session_id" | "turn_id"
>;

export type RetryNotice = {
  attempt: number;
  max: number;
  kind: FailureKind;
  delayMs: number;
};

function fallbackModel(
  info: Omit<ModelInfo, "supports_effort" | "api_model"> & { api_model?: string },
): ModelInfo {
  return {
    ...info,
    api_model: info.api_model ?? info.id,
    supports_effort: info.thinking_efforts.length > 0,
  };
}

export const MODEL_OPTIONS: ModelInfo[] = [
  fallbackModel({
    id: "deepseek-flash",
    label: "DeepSeek Flash",
    provider: "deepseek",
    api_model: "deepseek-flash",
    supports_vision: true,
    selectable: true,
    availability: "available",
    supports_thinking_toggle: true,
    thinking_efforts: ["low", "high", "max"],
    default_thinking_enabled: true,
    default_thinking_effort: "high",
    max_context: 1_000_000,
    max_output_tokens: 393_216,
  }),
  fallbackModel({
    id: "mimo-v2.5",
    label: "MiMo v2.5",
    provider: "mimo",
    supports_vision: true,
    selectable: true,
    availability: "available",
    supports_thinking_toggle: true,
    thinking_efforts: [],
    default_thinking_enabled: true,
    default_thinking_effort: "high",
    max_context: 1_000_000,
    max_output_tokens: 131_072,
  }),
  fallbackModel({
    id: "mimo-v2.5-pro",
    label: "MiMo v2.5 Pro",
    provider: "mimo",
    supports_vision: false,
    selectable: true,
    availability: "available",
    supports_thinking_toggle: true,
    thinking_efforts: [],
    default_thinking_enabled: true,
    default_thinking_effort: "high",
    max_context: 1_000_000,
    max_output_tokens: 131_072,
  }),
  fallbackModel({
    id: "kimi-k3",
    label: "Kimi K3",
    provider: "kimi",
    supports_vision: true,
    selectable: true,
    availability: "available",
    supports_thinking_toggle: false,
    thinking_efforts: ["low", "high", "max"],
    default_thinking_enabled: true,
    default_thinking_effort: "max",
    max_context: 1_000_000,
    max_output_tokens: 1_048_576,
  }),
  fallbackModel({
    id: "gemini-3.8-flash",
    label: "Gemini 3.8 Flash",
    provider: "google",
    supports_vision: true,
    selectable: true,
    availability: "available",
    supports_thinking_toggle: false,
    thinking_efforts: ["low", "medium", "high"],
    default_thinking_enabled: true,
    default_thinking_effort: "medium",
    max_context: 1_048_576,
    max_output_tokens: 65_536,
  }),
  fallbackModel({
    id: "glm-5.3-flash",
    label: "GLM-5.3-Flash",
    provider: "zhipu",
    supports_vision: true,
    selectable: true,
    availability: "available",
    supports_thinking_toggle: false,
    thinking_efforts: ["low", "high", "max"],
    default_thinking_enabled: true,
    default_thinking_effort: "max",
    max_context: 1_000_000,
    max_output_tokens: 131_072,
  }),
  fallbackModel({
    id: "deepseek-v4-pro",
    label: "DeepSeek V4 Pro",
    provider: "deepseek",
    supports_vision: false,
    selectable: false,
    availability: "available",
    supports_thinking_toggle: true,
    thinking_efforts: ["high", "max"],
    default_thinking_enabled: true,
    default_thinking_effort: "high",
    max_context: 1_000_000,
    max_output_tokens: 393_216,
  }),
  fallbackModel({
    id: "kimi-k2.6",
    label: "Kimi K2.6",
    provider: "kimi",
    supports_vision: true,
    selectable: false,
    availability: "available",
    supports_thinking_toggle: true,
    thinking_efforts: [],
    default_thinking_enabled: true,
    default_thinking_effort: "high",
    max_context: 256_000,
  }),
  fallbackModel({
    id: "mimo-v2.5-pro-ultraspeed",
    label: "MiMo v2.5 Pro Ultraspeed",
    provider: "mimo",
    supports_vision: false,
    selectable: false,
    availability: "retired",
    supports_thinking_toggle: true,
    thinking_efforts: [],
    default_thinking_enabled: true,
    default_thinking_effort: "high",
    max_context: 1_000_000,
    max_output_tokens: 131_072,
  }),
];

export const API_PROVIDERS: string[] = [...new Set(MODEL_OPTIONS.map((m) => m.provider))];

export function selectableModels(models: ModelInfo[] = MODEL_OPTIONS): ModelInfo[] {
  return models.filter((model) => model.selectable);
}

export function canonicalModelId(id: string): string {
  return id === "deepseek-v4-flash" ? "deepseek-flash" : id;
}

export function findModel(models: ModelInfo[], id: string): ModelInfo | undefined {
  const canonical = canonicalModelId(id);
  return models.find((model) => model.id === canonical)
    ?? MODEL_OPTIONS.find((model) => model.id === canonical);
}

export function formatModelThinkingLabel(model: ModelInfo | undefined, config: { model: string; thinking_enabled: boolean; thinking_effort: string }): string {
  const name = model?.label ?? config.model;
  if (model?.availability === "retired") {
    return `${name} · 已不可调用`;
  }
  if (model?.supports_thinking_toggle && !config.thinking_enabled) {
    return `${name} · 思考关闭`;
  }
  if (model?.thinking_efforts?.length) {
    return `${name} · ${config.thinking_effort}`;
  }
  return name;
}

export function providerLabel(provider: string): string {
  switch (provider) {
    case "deepseek":
      return "DeepSeek";
    case "kimi":
      return "Kimi";
    case "mimo":
      return "MiMo";
    case "google":
      return "Google Gemini";
    case "zhipu":
      return "智谱 GLM";
    default:
      return provider;
  }
}
