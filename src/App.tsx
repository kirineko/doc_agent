import { useEffect, useMemo, useReducer, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ChatPanel } from "./components/ChatPanel";
import { Sidebar } from "./components/Sidebar";
import { ToolChainPanel } from "./components/ToolChainPanel";
import {
  AgentStreamState,
  applyAgentEvent,
  initialAgentStreamState,
  markAgentBusy,
} from "./lib/agentEvents";
import { AgentEvent, Message, MessageBundle, Project, Session } from "./types";

type StreamAction =
  | { type: "event"; event: AgentEvent; sessionId?: string }
  | { type: "busy" };

function streamReducer(state: AgentStreamState, action: StreamAction): AgentStreamState {
  if (action.type === "busy") {
    return markAgentBusy(state);
  }
  return applyAgentEvent(state, action.event, action.sessionId);
}

function createOptimisticUserMessage(sessionId: string, content: string): Message {
  return {
    id: `pending-${crypto.randomUUID()}`,
    session_id: sessionId,
    role: "user",
    content,
    reasoning_content: null,
    tool_call_id: null,
    seq: Number.MAX_SAFE_INTEGER,
    created_at: new Date().toISOString(),
  };
}

function App() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [sessions, setSessions] = useState<Session[]>([]);
  const [messages, setMessages] = useState<Message[]>([]);
  const [activeProjectId, setActiveProjectId] = useState<string>();
  const [activeSessionId, setActiveSessionId] = useState<string>();
  const [input, setInput] = useState("");
  const [stream, dispatchStream] = useReducer(streamReducer, initialAgentStreamState);
  const [apiKeyStatus, setApiKeyStatus] = useState<Record<string, boolean>>({});
  const sendingRef = useRef(false);

  useEffect(() => {
    invoke<Project[]>("list_projects").then(setProjects).catch(console.error);
    ["deepseek", "kimi"].forEach(async (provider) => {
      const has = await invoke<boolean>("has_api_key", { provider });
      setApiKeyStatus((prev) => ({ ...prev, [provider]: has }));
    });
  }, []);

  useEffect(() => {
    if (!activeProjectId) {
      setSessions([]);
      return;
    }
    invoke<Session[]>("list_sessions", { projectId: activeProjectId })
      .then(setSessions)
      .catch(console.error);
  }, [activeProjectId]);

  useEffect(() => {
    if (!activeSessionId) {
      setMessages([]);
      return;
    }
    invoke<MessageBundle>("list_messages", { sessionId: activeSessionId })
      .then((bundle) => setMessages(bundle.messages))
      .catch(console.error);
  }, [activeSessionId]);

  useEffect(() => {
    const unlisten = listen<AgentEvent>("agent-event", (event) => {
      const payload = event.payload;
      dispatchStream({ type: "event", event: payload, sessionId: activeSessionId });
      if (payload.kind === "turn_complete" && activeSessionId) {
        invoke<MessageBundle>("list_messages", { sessionId: activeSessionId })
          .then((bundle) => setMessages(bundle.messages))
          .catch(console.error);
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [activeSessionId]);

  const activeProjectName = useMemo(
    () => projects.find((p) => p.id === activeProjectId)?.name,
    [projects, activeProjectId],
  );

  async function reloadMessages(sessionId: string) {
    const bundle = await invoke<MessageBundle>("list_messages", { sessionId });
    setMessages(bundle.messages);
  }

  async function sendMessage() {
    const content = input.trim();
    if (!activeSessionId || !content || stream.busy || sendingRef.current) {
      return;
    }

    sendingRef.current = true;
    setInput("");
    dispatchStream({ type: "busy" });
    setMessages((prev) => [...prev, createOptimisticUserMessage(activeSessionId, content)]);

    try {
      await invoke("send_message", {
        req: { session_id: activeSessionId, content },
      });
    } catch (error) {
      console.error(error);
      await reloadMessages(activeSessionId).catch(console.error);
      dispatchStream({
        type: "event",
        event: {
          kind: "error",
          session_id: activeSessionId,
          turn_id: "local",
          message: String(error),
        },
        sessionId: activeSessionId,
      });
    } finally {
      sendingRef.current = false;
    }
  }

  return (
    <div className="flex h-full flex-col bg-[#0b1020]">
      <header className="flex items-center gap-3 border-b border-slate-800 px-3 py-1.5">
        <div className="text-sm font-semibold text-white">Doc Agent</div>
        <div className="truncate text-xs text-slate-400">
          {activeProjectName ? activeProjectName : "请选择项目目录"}
        </div>
      </header>
      <main className="flex min-h-0 flex-1 gap-2.5 p-2.5">
        <Sidebar
          projects={projects}
          sessions={sessions}
          activeProjectId={activeProjectId}
          activeSessionId={activeSessionId}
          apiKeyStatus={apiKeyStatus}
          onProjectsChange={setProjects}
          onSessionsChange={setSessions}
          onSelectProject={setActiveProjectId}
          onSelectSession={setActiveSessionId}
          onSessionUpdated={(session) =>
            setSessions((prev) => prev.map((item) => (item.id === session.id ? session : item)))
          }
          onApiKeyStatusChange={(provider, has) =>
            setApiKeyStatus((prev) => ({ ...prev, [provider]: has }))
          }
        />
        <ChatPanel
          messages={messages}
          streamingReasoning={stream.streamingReasoning}
          streamingContent={stream.streamingContent}
          input={input}
          busy={stream.busy}
          onInputChange={setInput}
          onSend={sendMessage}
        />
        <ToolChainPanel items={stream.liveTools} />
      </main>
    </div>
  );
}

export default App;
