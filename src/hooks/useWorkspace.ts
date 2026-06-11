import { useCallback, useEffect, useMemo, useReducer, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { formatCharCount } from "../components/ToolChainPanel";
import { toolLabel } from "../lib/toolLabels";
import {
  countChatMessages,
  isStaleSessionResult,
  shouldDiscardFollowup,
  canRequestStarter,
} from "../lib/suggestions";
import { appendAssistantStepDone } from "../lib/messages";
import {
  mostRecentSessionId,
  shouldApplyProjectSelection,
} from "../lib/projectSession";
import {
  buildCreateSessionRequest,
  DEFAULT_SESSION_CONFIG,
  isSessionModelLocked,
  type SessionConfig,
} from "../lib/sessionConfig";
import { getSendBlocker, type SendBlocker } from "../lib/sendReadiness";
import { createOptimisticUserMessage } from "../lib/workspaceMessages";
import { initialAgentStreamState, streamReducer } from "../lib/workspaceStream";
import { useProjectFiles } from "./useProjectFiles";
import { API_PROVIDERS, type AgentEvent, type Message, type MessageBundle, type Project, type Session } from "../types";

function clearSendHighlights(
  setHighlightProject: (value: boolean) => void,
  setHighlightApiKeyProvider: (value: string | undefined) => void,
) {
  setHighlightProject(false);
  setHighlightApiKeyProvider(undefined);
}

export function useWorkspace() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [sessions, setSessions] = useState<Session[]>([]);
  const [messages, setMessages] = useState<Message[]>([]);
  const [messagesLoaded, setMessagesLoaded] = useState(true);
  const [activeProjectId, setActiveProjectId] = useState<string>();
  const [activeSessionId, setActiveSessionId] = useState<string>();
  const [pendingSessionConfig, setPendingSessionConfig] =
    useState<SessionConfig>(DEFAULT_SESSION_CONFIG);
  const [input, setInput] = useState("");
  const [stream, dispatchStream] = useReducer(streamReducer, initialAgentStreamState);
  const [apiKeyStatus, setApiKeyStatus] = useState<Record<string, boolean>>({});
  const [tavilyEnabled, setTavilyEnabled] = useState(false);
  const [initializing, setInitializing] = useState(false);
  const [starterSuggestions, setStarterSuggestions] = useState<string[]>([]);
  const [followupSuggestions, setFollowupSuggestions] = useState<string[]>([]);
  const projectFiles = useProjectFiles(activeProjectId);
  const [sendHint, setSendHint] = useState<SendBlocker | null>(null);
  const [highlightProject, setHighlightProject] = useState(false);
  const [highlightApiKeyProvider, setHighlightApiKeyProvider] = useState<string>();
  const sendingRef = useRef(false);
  const initStarterInFlightRef = useRef(false);
  const ensureSessionInFlightRef = useRef<Promise<string | null> | null>(null);
  const selectionTargetProjectIdRef = useRef<string | undefined>(undefined);
  const activeSessionRef = useRef<string | undefined>(undefined);
  const activeProjectRef = useRef<string | undefined>(undefined);
  const messagesRef = useRef<Message[]>([]);
  const sessionsRef = useRef<Session[]>([]);
  const pendingSessionConfigRef = useRef(pendingSessionConfig);
  const starterStartedRef = useRef<string | undefined>(undefined);
  const skipNextLoadRef = useRef(false);
  const onProjectFilesAgentEventRef = useRef(projectFiles.onAgentEvent);
  onProjectFilesAgentEventRef.current = projectFiles.onAgentEvent;

  activeSessionRef.current = activeSessionId;
  activeProjectRef.current = activeProjectId;
  messagesRef.current = messages;
  sessionsRef.current = sessions;
  pendingSessionConfigRef.current = pendingSessionConfig;

  useEffect(() => {
    invoke<Project[]>("list_projects").then(setProjects).catch(console.error);
    API_PROVIDERS.forEach(async (provider) => {
      const has = await invoke<boolean>("has_api_key", { provider });
      setApiKeyStatus((prev) => ({ ...prev, [provider]: has }));
    });
    invoke<boolean>("has_api_key", { provider: "tavily" })
      .then(setTavilyEnabled)
      .catch(console.error);
  }, []);

  const selectProject = useCallback(async (projectId: string | undefined) => {
    if (projectId && projectId === activeProjectRef.current) return;

    selectionTargetProjectIdRef.current = projectId;
    setActiveProjectId(projectId);
    setHighlightProject(false);

    if (!projectId) {
      setSessions([]);
      setActiveSessionId(undefined);
      projectFiles.reset();
      return;
    }

    try {
      const list = await invoke<Session[]>("list_sessions", { projectId });
      if (!shouldApplyProjectSelection(projectId, selectionTargetProjectIdRef.current)) return;
      setSessions(list);
      void projectFiles.loadInitial(projectId);
      setActiveSessionId(mostRecentSessionId(list));
    } catch (error) {
      console.error(error);
    }
  }, [projectFiles.loadInitial, projectFiles.reset]);

  useEffect(() => {
    if (!activeSessionId) {
      setMessages([]);
      setMessagesLoaded(true);
      setStarterSuggestions([]);
      setFollowupSuggestions([]);
      setInitializing(false);
      starterStartedRef.current = undefined;
      return;
    }
    if (skipNextLoadRef.current) {
      skipNextLoadRef.current = false;
      setMessagesLoaded(true);
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
      .catch((error) => {
        console.error(error);
        if (!cancelled) setMessagesLoaded(true);
      });

    return () => {
      cancelled = true;
    };
  }, [activeSessionId]);

  const activeSession = useMemo(
    () => sessions.find((s) => s.id === activeSessionId),
    [sessions, activeSessionId],
  );

  const chatMessageCount = useMemo(() => countChatMessages(messages), [messages]);
  const sessionContextReady = !activeSessionId || messagesLoaded;
  const modelLocked =
    isSessionModelLocked(chatMessageCount) ||
    (Boolean(activeSessionId) && !messagesLoaded);

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
      // 静默失败：重置守卫，允许用户再次点击胶囊重试
      starterStartedRef.current = undefined;
    } finally {
      if (!isStaleSessionResult(sessionId, activeSessionRef.current)) {
        setInitializing(false);
      }
    }
  }, [apiKeyStatus.deepseek]);

  const runFollowup = useCallback(async (sessionId: string, messageCount: number) => {
    if (!apiKeyStatus.deepseek) return;
    try {
      const items = await invoke<string[]>("generate_suggestions", {
        req: { session_id: sessionId, kind: "followup" },
      });
      if (isStaleSessionResult(sessionId, activeSessionRef.current)) return;
      if (shouldDiscardFollowup(messageCount, countChatMessages(messagesRef.current))) return;
      setFollowupSuggestions(items);
    } catch {
      // 静默
    }
  }, [apiKeyStatus.deepseek]);

  useEffect(() => {
    const unlisten = listen<AgentEvent>("agent-event", (event) => {
      const payload = event.payload;
      dispatchStream({ type: "event", event: payload, sessionId: activeSessionRef.current });
      onProjectFilesAgentEventRef.current(payload);
      if (payload.kind === "assistant_step_done") {
        setMessages((prev) =>
          appendAssistantStepDone(prev, payload, activeSessionRef.current),
        );
      }
      if (payload.kind === "turn_complete" && activeSessionRef.current) {
        const sessionId = activeSessionRef.current;
        if (isStaleSessionResult(payload.session_id, sessionId)) return;
        invoke<MessageBundle>("list_messages", { sessionId })
          .then((bundle) => {
            setMessages(bundle.messages);
            setFollowupSuggestions([]);
            if (activeProjectRef.current) {
              invoke<Session[]>("list_sessions", { projectId: activeProjectRef.current })
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
  }, [runFollowup]);

  useEffect(() => {
    if (!sendHint) return;
    const timer = window.setTimeout(() => {
      setSendHint(null);
      clearSendHighlights(setHighlightProject, setHighlightApiKeyProvider);
    }, 4000);
    return () => window.clearTimeout(timer);
  }, [sendHint]);

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
    if (running) return `正在执行「${toolLabel(running.name)}」…`;
    return undefined;
  }, [stream.liveTools]);

  const showInitCapsule = Boolean(
    activeProjectId &&
      sessionContextReady &&
      apiKeyStatus.deepseek &&
      chatMessageCount === 0 &&
      !initializing &&
      !stream.busy &&
      starterSuggestions.length === 0,
  );

  async function reloadMessages(sessionId: string) {
    const bundle = await invoke<MessageBundle>("list_messages", { sessionId });
    setMessages(bundle.messages);
    setMessagesLoaded(true);
  }

  async function ensureSession(): Promise<string | null> {
    if (ensureSessionInFlightRef.current) return ensureSessionInFlightRef.current;

    const task = (async (): Promise<string | null> => {
      const projectId = activeProjectRef.current;
      if (!projectId) return null;

      const currentSessionId = activeSessionRef.current;
      if (
        currentSessionId &&
        sessionsRef.current.some((s) => s.id === currentSessionId && s.project_id === projectId)
      ) {
        return currentSessionId;
      }

      const session = await invoke<Session>("create_session", {
        req: buildCreateSessionRequest(projectId, pendingSessionConfigRef.current),
      });

      skipNextLoadRef.current = true;
      sessionsRef.current = [session, ...sessionsRef.current];
      activeSessionRef.current = session.id;
      setSessions(sessionsRef.current);
      setActiveSessionId(session.id);
      setMessages([]);
      setMessagesLoaded(true);
      starterStartedRef.current = undefined;
      return session.id;
    })();

    ensureSessionInFlightRef.current = task;
    try {
      return await task;
    } finally {
      if (ensureSessionInFlightRef.current === task) {
        ensureSessionInFlightRef.current = null;
      }
    }
  }

  function showSendBlocker(blocker: SendBlocker) {
    setSendHint(blocker);
    if (blocker.kind === "no_project") {
      setHighlightProject(true);
      document.getElementById("sidebar-projects")?.scrollIntoView({ block: "nearest" });
      return;
    }
    setHighlightApiKeyProvider(blocker.provider);
    requestAnimationFrame(() => {
      document.getElementById("sidebar-api-keys")?.scrollIntoView({ block: "nearest" });
    });
  }

  async function sendMessageContent(content: string) {
    const trimmed = content.trim();
    if (!trimmed || stream.busy || sendingRef.current) return;

    const model = activeSession?.model ?? pendingSessionConfigRef.current.model;
    const blocker = getSendBlocker({
      activeProjectId: activeProjectRef.current,
      model,
      apiKeyStatus,
    });
    if (blocker) {
      showSendBlocker(blocker);
      return;
    }

    sendingRef.current = true;
    try {
      const sessionId = await ensureSession();
      if (!sessionId) {
        showSendBlocker({ kind: "no_project" });
        return;
      }

      setInput("");
      setSendHint(null);
      clearSendHighlights(setHighlightProject, setHighlightApiKeyProvider);
      setStarterSuggestions([]);
      setFollowupSuggestions([]);
      starterStartedRef.current = undefined;
      dispatchStream({ type: "busy" });
      setMessages((prev) => [...prev, createOptimisticUserMessage(sessionId, trimmed)]);

      await invoke("send_message", { req: { session_id: sessionId, content: trimmed } });
    } catch (error) {
      console.error(error);
      const sessionId = activeSessionRef.current;
      if (sessionId) {
        await reloadMessages(sessionId).catch(console.error);
        dispatchStream({
          type: "event",
          event: {
            kind: "error",
            session_id: sessionId,
            turn_id: "local",
            message: String(error),
          },
          sessionId,
        });
      }
    } finally {
      sendingRef.current = false;
    }
  }

  async function sendMessage() {
    await sendMessageContent(input);
  }

  async function handleInitStarter() {
    if (
      !sessionContextReady ||
      !canRequestStarter(apiKeyStatus.deepseek, chatMessageCount, initializing) ||
      stream.busy ||
      initStarterInFlightRef.current
    ) {
      return;
    }

    initStarterInFlightRef.current = true;
    try {
      const sessionId = await ensureSession();
      if (!sessionId) {
        showSendBlocker({ kind: "no_project" });
        return;
      }
      await runStarter(sessionId);
    } finally {
      initStarterInFlightRef.current = false;
    }
  }

  const handleApiKeyStatusChange = useCallback((provider: string, has: boolean) => {
    setApiKeyStatus((prev) => ({ ...prev, [provider]: has }));
    if (has && sendHint?.kind === "no_api_key" && sendHint.provider === provider) {
      setSendHint(null);
      setHighlightApiKeyProvider(undefined);
    }
  }, [sendHint]);

  const handleTavilyStatusChange = useCallback((has: boolean) => {
    setTavilyEnabled(has);
  }, []);

  const handlePendingSessionConfigChange = useCallback((patch: Partial<SessionConfig>) => {
    setPendingSessionConfig((prev) => ({ ...prev, ...patch }));
  }, []);

  const dismissSendHint = useCallback(() => {
    setSendHint(null);
    clearSendHighlights(setHighlightProject, setHighlightApiKeyProvider);
  }, []);

  const handleSessionUpdated = useCallback((session: Session) => {
    setSessions((prev) => prev.map((item) => (item.id === session.id ? session : item)));
  }, []);

  return {
    projects,
    setProjects,
    sessions,
    setSessions,
    messages,
    activeProjectId,
    activeSessionId,
    setActiveSessionId,
    pendingSessionConfig,
    input,
    setInput,
    stream,
    apiKeyStatus,
    tavilyEnabled,
    initializing,
    starterSuggestions,
    followupSuggestions,
    filePaths: projectFiles.filePaths,
    fileRevision: projectFiles.fileRevision,
    sendHint,
    highlightProject,
    highlightApiKeyProvider,
    modelLocked,
    sessionContextReady,
    activeProjectName,
    activity,
    showInitCapsule,
    selectProject,
    sendMessage,
    handleInitStarter,
    handleApiKeyStatusChange,
    handleTavilyStatusChange,
    handlePendingSessionConfigChange,
    dismissSendHint,
    handleSessionUpdated,
  };
}
