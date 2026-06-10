import { useCallback, useEffect, useMemo, useReducer, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ChatPanel } from "./components/ChatPanel";
import { Sidebar } from "./components/Sidebar";
import { formatCharCount, ToolChainPanel } from "./components/ToolChainPanel";
import { toolLabel } from "./lib/toolLabels";
import {
  countChatMessages,
  isStaleSessionResult,
  shouldDiscardFollowup,
  shouldRunStarter,
} from "./lib/suggestions";
import {
  AgentStreamState,
  applyAgentEvent,
  initialAgentStreamState,
  markAgentBusy,
} from "./lib/agentEvents";
import {
  AgentEvent,
  Message,
  MessageBundle,
  Project,
  ProjectFileList,
  Session,
} from "./types";

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
  const [messagesLoaded, setMessagesLoaded] = useState(false);
  const [activeProjectId, setActiveProjectId] = useState<string>();
  const [activeSessionId, setActiveSessionId] = useState<string>();
  const [input, setInput] = useState("");
  const [stream, dispatchStream] = useReducer(streamReducer, initialAgentStreamState);
  const [apiKeyStatus, setApiKeyStatus] = useState<Record<string, boolean>>({});
  const [initializing, setInitializing] = useState(false);
  const [starterSuggestions, setStarterSuggestions] = useState<string[]>([]);
  const [followupSuggestions, setFollowupSuggestions] = useState<string[]>([]);
  const [filePaths, setFilePaths] = useState<string[]>([]);
  const sendingRef = useRef(false);
  const activeSessionRef = useRef<string | undefined>(undefined);
  const messagesRef = useRef<Message[]>([]);
  const starterStartedRef = useRef<string | undefined>(undefined);

  activeSessionRef.current = activeSessionId;
  messagesRef.current = messages;

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
      setFilePaths([]);
      return;
    }
    invoke<Session[]>("list_sessions", { projectId: activeProjectId })
      .then(setSessions)
      .catch(console.error);
    invoke<ProjectFileList>("list_project_files_cmd", { projectId: activeProjectId })
      .then((list) => setFilePaths(list.entries.map((e) => e.path)))
      .catch(console.error);
  }, [activeProjectId]);

  useEffect(() => {
    if (!activeSessionId) {
      setMessages([]);
      setMessagesLoaded(false);
      setStarterSuggestions([]);
      setFollowupSuggestions([]);
      setInitializing(false);
      starterStartedRef.current = undefined;
      return;
    }
    let cancelled = false;
    setMessages([]);
    setMessagesLoaded(false);
    setStarterSuggestions([]);
    setFollowupSuggestions([]);
    setInitializing(false);
    starterStartedRef.current = undefined;

    invoke<MessageBundle>("list_messages", { sessionId: activeSessionId })
      .then((bundle) => {
        if (cancelled) return;
        setMessages(bundle.messages);
        setMessagesLoaded(true);
      })
      .catch(console.error);

    return () => {
      cancelled = true;
    };
  }, [activeSessionId]);

  const chatMessageCount = useMemo(() => countChatMessages(messages), [messages]);

  const runStarter = useCallback(async (sessionId: string) => {
    if (!apiKeyStatus.deepseek) return;
    if (starterStartedRef.current === sessionId) return;
    starterStartedRef.current = sessionId;

    setInitializing(true);
    setStarterSuggestions([]);
    try {
      const items = await invoke<string[]>("generate_suggestions", {
        req: { session_id: sessionId, kind: "starter" },
      });
      if (isStaleSessionResult(sessionId, activeSessionRef.current)) return;
      if (countChatMessages(messagesRef.current) > 0) return;
      setStarterSuggestions(items);
    } catch {
      // key 未配置或调用失败：静默
    } finally {
      if (!isStaleSessionResult(sessionId, activeSessionRef.current)) {
        setInitializing(false);
      }
    }
  }, [apiKeyStatus.deepseek]);

  useEffect(() => {
    if (!activeSessionId || !messagesLoaded) return;

    if (!shouldRunStarter(apiKeyStatus.deepseek, chatMessageCount, initializing)) {
      if (chatMessageCount > 0) {
        setStarterSuggestions([]);
        setInitializing(false);
        starterStartedRef.current = undefined;
      }
      return;
    }

    void runStarter(activeSessionId);
  }, [
    activeSessionId,
    messagesLoaded,
    chatMessageCount,
    apiKeyStatus.deepseek,
    initializing,
    runStarter,
  ]);

  const runFollowup = useCallback(async (sessionId: string, messageCount: number) => {
    if (!apiKeyStatus.deepseek) return;
    try {
      const items = await invoke<string[]>("generate_suggestions", {
        req: { session_id: sessionId, kind: "followup" },
      });
      if (isStaleSessionResult(sessionId, activeSessionRef.current)) return;
      if (
        shouldDiscardFollowup(messageCount, countChatMessages(messagesRef.current))
      ) {
        return;
      }
      setFollowupSuggestions(items);
    } catch {
      // 静默
    }
  }, [apiKeyStatus.deepseek]);

  useEffect(() => {
    const unlisten = listen<AgentEvent>("agent-event", (event) => {
      const payload = event.payload;
      dispatchStream({ type: "event", event: payload, sessionId: activeSessionRef.current });
      if (payload.kind === "turn_complete" && activeSessionRef.current) {
        const sessionId = activeSessionRef.current;
        invoke<MessageBundle>("list_messages", { sessionId })
          .then((bundle) => {
            setMessages(bundle.messages);
            setFollowupSuggestions([]);
            if (activeProjectId) {
              invoke<Session[]>("list_sessions", { projectId: activeProjectId })
                .then(setSessions)
                .catch(console.error);
            }
            void runFollowup(sessionId, countChatMessages(bundle.messages));
          })
          .catch(console.error);
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [activeProjectId, runFollowup]);

  const activeProjectName = useMemo(
    () => projects.find((p) => p.id === activeProjectId)?.name,
    [projects, activeProjectId],
  );

  const activity = useMemo(() => {
    const streaming = stream.liveTools.find((t) => t.status === "streaming");
    if (streaming) {
      return `正在生成「${toolLabel(streaming.name)}」调用参数…（已接收 ${formatCharCount(streaming.argsChars ?? 0)}）`;
    }
    const running = stream.liveTools.find((t) => t.status === "running");
    if (running) {
      return `正在执行「${toolLabel(running.name)}」…`;
    }
    return undefined;
  }, [stream.liveTools]);

  async function reloadMessages(sessionId: string) {
    const bundle = await invoke<MessageBundle>("list_messages", { sessionId });
    setMessages(bundle.messages);
    setMessagesLoaded(true);
  }

  async function sendMessageContent(content: string) {
    if (!activeSessionId || !content || stream.busy || sendingRef.current) {
      return;
    }

    sendingRef.current = true;
    setInput("");
    setStarterSuggestions([]);
    setFollowupSuggestions([]);
    starterStartedRef.current = undefined;
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

  async function sendMessage() {
    await sendMessageContent(input.trim());
  }

  function handleApiKeyStatusChange(provider: string, has: boolean) {
    setApiKeyStatus((prev) => ({ ...prev, [provider]: has }));
    if (
      provider === "deepseek" &&
      has &&
      activeSessionId &&
      messagesLoaded &&
      shouldRunStarter(true, chatMessageCount, initializing)
    ) {
      starterStartedRef.current = undefined;
      void runStarter(activeSessionId);
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
          onApiKeyStatusChange={handleApiKeyStatusChange}
        />
        <ChatPanel
          sessionId={activeSessionId}
          messages={messages}
          streamingReasoning={stream.streamingReasoning}
          streamingContent={stream.streamingContent}
          activity={activity}
          initializing={initializing}
          starterSuggestions={starterSuggestions}
          followupSuggestions={followupSuggestions}
          filePaths={filePaths}
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
