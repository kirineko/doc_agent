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

export interface Message {
  id: string;
  session_id: string;
  role: string;
  content?: string | null;
  reasoning_content?: string | null;
  tool_call_id?: string | null;
  seq: number;
  created_at: string;
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
}

export type AgentEvent =
  | { kind: "reasoning_token"; session_id: string; turn_id: string; delta: string }
  | { kind: "content_token"; session_id: string; turn_id: string; delta: string }
  | { kind: "tool_call"; session_id: string; turn_id: string; id: string; name: string; args: unknown; status: string }
  | { kind: "tool_result"; session_id: string; turn_id: string; id: string; ok: boolean; summary: string; duration_ms: number }
  | { kind: "turn_complete"; session_id: string; turn_id: string }
  | { kind: "error"; session_id: string; turn_id: string; message: string };

export const MODEL_OPTIONS = [
  { id: "mock", label: "Mock（本地调试）", provider: "mock", supportsEffort: false },
  { id: "deepseek-v4-flash", label: "DeepSeek V4 Flash", provider: "deepseek", supportsEffort: true },
  { id: "deepseek-v4-pro", label: "DeepSeek V4 Pro", provider: "deepseek", supportsEffort: true },
  { id: "kimi-k2.6", label: "Kimi K2.6", provider: "kimi", supportsEffort: false },
] as const;

export function providerLabel(provider: string): string {
  switch (provider) {
    case "deepseek":
      return "DeepSeek";
    case "kimi":
      return "Kimi";
    default:
      return provider;
  }
}
