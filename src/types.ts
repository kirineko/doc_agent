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

export type ClarifyKind = "single" | "multi" | "text" | "confirm_brief";

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
}

export interface ClarifyAnswer {
  question_id: string;
  selected: string[];
  custom?: string | null;
  display_text: string;
  brief?: Record<string, string> | null;
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

export interface ModelInfo {
  id: string;
  label: string;
  provider: string;
  api_model: string;
  supports_vision: boolean;
  supports_effort: boolean;
  max_context: number;
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
    }
  | { kind: "error"; session_id: string; turn_id: string; message: string }
  | { kind: "session_title_updated"; session_id: string; title: string };

export const MODEL_OPTIONS = [
  { id: "deepseek-v4-flash", label: "DeepSeek V4 Flash", provider: "deepseek", supportsEffort: true, supportsVision: false },
  { id: "deepseek-v4-pro", label: "DeepSeek V4 Pro", provider: "deepseek", supportsEffort: true, supportsVision: false },
  { id: "kimi-k2.6", label: "Kimi K2.6", provider: "kimi", supportsEffort: false, supportsVision: true },
  { id: "mimo-v2.5", label: "MiMo v2.5", provider: "mimo", supportsEffort: false, supportsVision: true },
  { id: "mimo-v2.5-pro", label: "MiMo v2.5 Pro", provider: "mimo", supportsEffort: false, supportsVision: false },
  { id: "mimo-v2.5-pro-ultraspeed", label: "MiMo v2.5 Pro Ultraspeed", provider: "mimo", supportsEffort: false, supportsVision: false },
] as const;

export const API_PROVIDERS: string[] = [...new Set(MODEL_OPTIONS.map((m) => m.provider))];

export function providerLabel(provider: string): string {
  switch (provider) {
    case "deepseek":
      return "DeepSeek";
    case "kimi":
      return "Kimi";
    case "mimo":
      return "MiMo";
    default:
      return provider;
  }
}
