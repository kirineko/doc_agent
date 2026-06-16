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
  displaySessionsForProject,
  moveSessionInList,
  prependSessionToOrder,
  removeSessionFromOrder,
  writeProjectOrder,
} from "../lib/sessionOrder";
import {
  buildCreateSessionRequest,
  isSessionModelLocked,
  readStoredSessionConfig,
  resolveSessionConfig,
  sessionConfigFromSession,
  writeStoredSessionConfig,
  type SessionConfig,
} from "../lib/sessionConfig";
import { getSendBlocker, type SendBlocker } from "../lib/sendReadiness";
import { refreshWebSearchState, setWebSearchActive as persistWebSearchActive } from "../lib/webSearch";
import { createOptimisticUserMessage } from "../lib/workspaceMessages";
import { initialAgentStreamState, streamReducer } from "../lib/workspaceStream";
import { useProjectFiles } from "./useProjectFiles";
import {
  type ActiveClarify,
  messageBundleState,
  parseClarifyQuestion,
} from "../lib/clarifyBrief";
import { loadModels, modelSupportsVision } from "../lib/models";
import {
  blobToBase64,
  extensionForMime,
  MAX_ATTACHMENTS_PER_MESSAGE,
  type PendingAttachment,
  revokePendingAttachments,
  userMessageWasPersisted,
} from "../lib/attachments";
import {
  API_PROVIDERS,
  type AgentEvent,
  type Message,
  type MessageBundle,
  type MessageAttachment,
  type ModelInfo,
  type Project,
  type Session,
  type ToolCallRecord,
} from "../types";

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
  const [toolCalls, setToolCalls] = useState<ToolCallRecord[]>([]);
  const [activeClarify, setActiveClarify] = useState<ActiveClarify>();
  const [messagesLoaded, setMessagesLoaded] = useState(true);
  const [activeProjectId, setActiveProjectId] = useState<string>();
  const [activeSessionId, setActiveSessionId] = useState<string>();
  const [pendingSessionConfig, setPendingSessionConfig] = useState<SessionConfig>(() =>
    readStoredSessionConfig(),
  );
  const [input, setInput] = useState("");
  const [pendingAttachments, setPendingAttachments] = useState<PendingAttachment[]>([]);
  const [visionToast, setVisionToast] = useState<string | null>(null);
  const [contextRatio, setContextRatio] = useState(0);
  const [stream, dispatchStream] = useReducer(streamReducer, initialAgentStreamState);
  const [apiKeyStatus, setApiKeyStatus] = useState<Record<string, boolean>>({});
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [tavilyHasKey, setTavilyHasKey] = useState(false);
  const [webSearchActive, setWebSearchActive] = useState(false);
  const [initializing, setInitializing] = useState(false);
  const [starterSuggestions, setStarterSuggestions] = useState<string[]>([]);
  const [followupSuggestions, setFollowupSuggestions] = useState<string[]>([]);
  const projectFiles = useProjectFiles(activeProjectId);
  const [sendHint, setSendHint] = useState<SendBlocker | null>(null);
  const [highlightProject, setHighlightProject] = useState(false);
  const [highlightApiKeyProvider, setHighlightApiKeyProvider] = useState<string>();
  const [credentialsOpen, setCredentialsOpen] = useState(false);
  const [credentialsHintDismissed, setCredentialsHintDismissed] = useState(false);
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
  const pendingAttachmentsRef = useRef(pendingAttachments);
  const onProjectFilesAgentEventRef = useRef(projectFiles.onAgentEvent);
  onProjectFilesAgentEventRef.current = projectFiles.onAgentEvent;

  activeSessionRef.current = activeSessionId;
  activeProjectRef.current = activeProjectId;
  messagesRef.current = messages;
  sessionsRef.current = sessions;
  pendingSessionConfigRef.current = pendingSessionConfig;
  pendingAttachmentsRef.current = pendingAttachments;

  function applyBundle(bundle: MessageBundle) {
    const next = messageBundleState(bundle);
    setMessages(next.messages);
    setToolCalls(next.toolCalls);
    setActiveClarify(next.activeClarify);
  }

  function clearSuggestions() {
    setStarterSuggestions([]);
    setFollowupSuggestions([]);
  }

  const setSessionsFromBackend = useCallback((list: Session[], projectId: string) => {
    const ordered = displaySessionsForProject(list, projectId);
    sessionsRef.current = ordered;
    setSessions(ordered);
  }, []);

  useEffect(() => {
    invoke<Project[]>("list_projects").then(setProjects).catch(console.error);
    loadModels().then(setModels).catch(console.error);
    API_PROVIDERS.forEach(async (provider) => {
      const has = await invoke<boolean>("has_api_key", { provider });
      setApiKeyStatus((prev) => ({ ...prev, [provider]: has }));
    });
    void refreshWebSearchState()
      .then(({ hasKey, active }) => {
        setTavilyHasKey(hasKey);
        setWebSearchActive(active);
      })
      .catch(console.error);
  }, []);

  useEffect(() => {
    if (models.length === 0) return;
    setPendingSessionConfig((prev) =>
      resolveSessionConfig(
        prev,
        models.map((model) => model.id),
      ),
    );
  }, [models]);

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
      setSessionsFromBackend(list, projectId);
      void projectFiles.loadInitial(projectId);
      setActiveSessionId(mostRecentSessionId(list));
    } catch (error) {
      console.error(error);
    }
  }, [projectFiles.loadInitial, projectFiles.reset, setSessionsFromBackend]);

  useEffect(() => {
    dispatchStream({ type: "reset" });

    if (!activeSessionId) {
      setContextRatio(0);
      setMessages([]);
      setToolCalls([]);
      setActiveClarify(undefined);
      setMessagesLoaded(true);
      setStarterSuggestions([]);
      setFollowupSuggestions([]);
      setInitializing(false);
      starterStartedRef.current = undefined;
      return;
    }

    let cancelled = false;
    invoke<{ ratio: number }>("get_session_context_usage", { sessionId: activeSessionId })
      .then((usage) => {
        if (!cancelled) setContextRatio(usage.ratio);
      })
      .catch(() => {
        if (!cancelled) setContextRatio(0);
      });

    if (skipNextLoadRef.current) {
      skipNextLoadRef.current = false;
      setMessagesLoaded(true);
      return () => {
        cancelled = true;
      };
    }

    setMessages([]);
    setToolCalls([]);
    setActiveClarify(undefined);
    setMessagesLoaded(false);
    setStarterSuggestions([]);
    setFollowupSuggestions([]);
    setInitializing(false);
    starterStartedRef.current = undefined;

    invoke<MessageBundle>("list_messages", { sessionId: activeSessionId })
      .then((bundle) => {
        if (cancelled) return;
        applyBundle(bundle);
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

  useEffect(() => {
    setPendingAttachments((prev) => {
      revokePendingAttachments(prev);
      return [];
    });
    setVisionToast(null);
  }, [activeSessionId, activeProjectId]);

  const activeSession = useMemo(
    () => sessions.find((s) => s.id === activeSessionId),
    [sessions, activeSessionId],
  );

  const chatMessageCount = useMemo(() => countChatMessages(messages), [messages]);
  const sessionContextReady = !activeSessionId || messagesLoaded;
  const modelLocked =
    isSessionModelLocked(chatMessageCount) ||
    (Boolean(activeSessionId) && !messagesLoaded);

  const effectiveSessionConfig: SessionConfig = activeSession
    ? {
        model: activeSession.model,
        thinking_enabled: activeSession.thinking_enabled,
        thinking_effort: activeSession.thinking_effort,
      }
    : pendingSessionConfig;

  const modelSummary = useMemo(() => {
    const model = models.find((m) => m.id === effectiveSessionConfig.model);
    const name = model?.label ?? effectiveSessionConfig.model;
    if (!effectiveSessionConfig.thinking_enabled) return `${name} · 思考关闭`;
    if (model?.supports_effort) return `${name} · ${effectiveSessionConfig.thinking_effort}`;
    return name;
  }, [models, effectiveSessionConfig]);

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
      if (payload.kind === "context_usage" && payload.session_id === activeSessionRef.current) {
        setContextRatio(payload.ratio);
      }
      onProjectFilesAgentEventRef.current(payload);
      if (payload.kind === "assistant_step_done") {
        setMessages((prev) =>
          appendAssistantStepDone(prev, payload, activeSessionRef.current),
        );
      }
      if (payload.kind === "clarify_question") {
        if (isStaleSessionResult(payload.session_id, activeSessionRef.current)) return;
        setActiveClarify({
          toolCallId: payload.tool_call_id,
          question: parseClarifyQuestion(payload.question) ?? payload.question,
        });
        clearSuggestions();
      }
      if (payload.kind === "tool_result") {
        if (isStaleSessionResult(payload.session_id, activeSessionRef.current)) return;
        setActiveClarify((prev) =>
          prev?.toolCallId === payload.id ? undefined : prev,
        );
      }
      if (payload.kind === "turn_awaiting_user") {
        if (isStaleSessionResult(payload.session_id, activeSessionRef.current)) return;
        clearSuggestions();
      }
      if (payload.kind === "context_compacted" && activeSessionRef.current) {
        const sessionId = activeSessionRef.current;
        if (isStaleSessionResult(payload.session_id, sessionId)) return;
        invoke<{ ratio: number }>("get_session_context_usage", { sessionId })
          .then((usage) => setContextRatio(usage.ratio))
          .catch(console.error);
        invoke<MessageBundle>("list_messages", { sessionId })
          .then((bundle) => applyBundle(bundle))
          .catch(console.error);
      }
      if (payload.kind === "turn_complete" && activeSessionRef.current) {
        const sessionId = activeSessionRef.current;
        if (isStaleSessionResult(payload.session_id, sessionId)) return;
        invoke<MessageBundle>("list_messages", { sessionId })
          .then((bundle) => {
            applyBundle(bundle);
            clearSuggestions();
            if (activeProjectRef.current) {
              const projectId = activeProjectRef.current;
              invoke<Session[]>("list_sessions", { projectId })
                .then((list) => setSessionsFromBackend(list, projectId))
                .catch(console.error);
            }
            void runFollowup(sessionId, countChatMessages(bundle.messages));
          })
          .catch(console.error);
      }
      if (payload.kind === "session_title_updated" && activeProjectRef.current) {
        const projectId = activeProjectRef.current;
        setSessions((prev) =>
          prev.map((s) =>
            s.id === payload.session_id ? { ...s, title: payload.title } : s,
          ),
        );
        invoke<Session[]>("list_sessions", { projectId })
          .then((list) => setSessionsFromBackend(list, projectId))
          .catch(console.error);
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [runFollowup, setSessionsFromBackend]);

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

  const activeProjectRoot = useMemo(
    () => projects.find((p) => p.id === activeProjectId)?.root_path,
    [projects, activeProjectId],
  );

  const supportsVision = useMemo(
    () => modelSupportsVision(models, effectiveSessionConfig.model),
    [models, effectiveSessionConfig.model],
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
    applyBundle(bundle);
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
      prependSessionToOrder(projectId, session.id);
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

  const clearPendingAttachmentsForModel = useCallback((modelId: string) => {
    if (modelSupportsVision(models, modelId)) return;
    if (pendingAttachmentsRef.current.length === 0) return;
    setPendingAttachments((prev) => {
      revokePendingAttachments(prev);
      return [];
    });
    setVisionToast("当前模型不支持图片输入，已移除待发送图片");
  }, [models]);

  function showSendBlocker(blocker: SendBlocker) {
    setSendHint(blocker);
    if (blocker.kind === "no_project") {
      setHighlightProject(true);
      document.getElementById("sidebar-projects")?.scrollIntoView({ block: "nearest" });
      return;
    }
    setHighlightApiKeyProvider(blocker.provider);
    setCredentialsOpen(true);
  }

  async function sendMessageContent(content: string) {
    const trimmed = content.trim();
    const attachments: MessageAttachment[] = pendingAttachments.map(({ path, mime }) => ({
      path,
      mime,
    }));
    if ((!trimmed && attachments.length === 0) || stream.busy || sendingRef.current) return;
    if (activeClarify) return;

    const model = activeSession?.model ?? pendingSessionConfigRef.current.model;
    if (attachments.length > 0 && !modelSupportsVision(models, model)) {
      setVisionToast("当前模型不支持图片输入，请选用 Kimi K2.6 或 MiMo v2.5");
      return;
    }
    const blocker = getSendBlocker({
      activeProjectId: activeProjectRef.current,
      model,
      apiKeyStatus,
      models,
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
      setMessages((prev) => [
        ...prev,
        createOptimisticUserMessage(sessionId, trimmed, attachments),
      ]);

      await invoke("send_message", {
        req: { session_id: sessionId, content: trimmed, attachments },
      });
      setPendingAttachments((prev) => {
        revokePendingAttachments(prev);
        return [];
      });
    } catch (error) {
      console.error(error);
      const sessionId = activeSessionRef.current;
      if (sessionId) {
        const bundle = await invoke<MessageBundle>("list_messages", { sessionId }).catch(() => null);
        if (
          bundle &&
          userMessageWasPersisted(bundle.messages, trimmed, attachments)
        ) {
          setPendingAttachments((prev) => {
            revokePendingAttachments(prev);
            return [];
          });
        }
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

  const addPastedImage = useCallback(
    async (file: File, mime: string) => {
      if (!modelSupportsVision(models, effectiveSessionConfig.model)) {
        setVisionToast("当前模型不支持图片输入，请选用 Kimi K2.6 或 MiMo v2.5");
        return;
      }
      if (!activeProjectRef.current) {
        showSendBlocker({ kind: "no_project" });
        return;
      }
      if (pendingAttachments.length >= MAX_ATTACHMENTS_PER_MESSAGE) {
        setVisionToast(`单条消息最多 ${MAX_ATTACHMENTS_PER_MESSAGE} 张图片`);
        return;
      }
      try {
        const data_base64 = await blobToBase64(file);
        const saved = await invoke<{ path: string; mime: string }>("save_upload", {
          req: {
            project_id: activeProjectRef.current,
            filename: `paste.${extensionForMime(mime)}`,
            mime,
            data_base64,
          },
        });
        const previewUrl = URL.createObjectURL(file);
        setPendingAttachments((prev) => [
          ...prev,
          { path: saved.path, mime: saved.mime, previewUrl },
        ]);
      } catch (error) {
        console.error(error);
        setVisionToast(String(error));
      }
    },
    [effectiveSessionConfig.model, models, pendingAttachments.length],
  );

  const removePendingAttachment = useCallback((path: string) => {
    setPendingAttachments((prev) => {
      const target = prev.find((item) => item.path === path);
      if (target) URL.revokeObjectURL(target.previewUrl);
      return prev.filter((item) => item.path !== path);
    });
  }, []);

  const dismissVisionToast = useCallback(() => {
    setVisionToast(null);
  }, []);

  async function submitClarifyAnswer(payload: { selected: string[]; custom?: string | null }) {
    if (!activeSessionRef.current || !activeClarify) return;
    const sessionId = activeSessionRef.current;
    const questionId = activeClarify.question.id;
    const previousActive = activeClarify;
    // 提交后立即收起底部卡片；若 resume 又触发新 clarify，由 clarify_question 事件恢复
    setActiveClarify(undefined);
    dispatchStream({ type: "busy_resume" });
    try {
      await invoke("submit_clarify_answer", {
        req: {
          session_id: sessionId,
          question_id: questionId,
          selected: payload.selected,
          custom: payload.custom ?? null,
        },
      });
    } catch (error) {
      console.error(error);
      setActiveClarify(previousActive);
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

  const handleTavilyKeyChange = useCallback(async (has: boolean) => {
    setTavilyHasKey(has);
    try {
      const { active } = await refreshWebSearchState();
      setWebSearchActive(active);
    } catch (error) {
      console.error(error);
      setWebSearchActive(has);
    }
  }, []);

  const enableWebSearch = useCallback(async () => {
    try {
      const { hasKey } = await refreshWebSearchState();
      setTavilyHasKey(hasKey);
      if (!hasKey) {
        setHighlightApiKeyProvider("tavily");
        setCredentialsOpen(true);
        return;
      }
      await persistWebSearchActive(true);
      setWebSearchActive(true);
    } catch (error) {
      console.error(error);
    }
  }, []);

  const disableWebSearch = useCallback(async () => {
    try {
      await persistWebSearchActive(false);
      setWebSearchActive(false);
    } catch (error) {
      console.error(error);
    }
  }, []);

  const handlePendingSessionConfigChange = useCallback(
    (patch: Partial<SessionConfig>) => {
      if (patch.model) {
        clearPendingAttachmentsForModel(patch.model);
      }
      setPendingSessionConfig((prev) => {
        const next = { ...prev, ...patch };
        writeStoredSessionConfig(next);
        return next;
      });
    },
    [clearPendingAttachmentsForModel],
  );

  const dismissSendHint = useCallback(() => {
    setSendHint(null);
    clearSendHighlights(setHighlightProject, setHighlightApiKeyProvider);
  }, []);

  const dismissCompactionNotice = useCallback(() => {
    dispatchStream({ type: "clear_compaction_notice" });
  }, []);

  const handleSessionUpdated = useCallback((session: Session) => {
    setSessions((prev) => {
      const next = prev.map((item) => (item.id === session.id ? session : item));
      sessionsRef.current = next;
      return next;
    });
  }, []);

  const updateSessionConfig = useCallback(
    async (patch: Partial<SessionConfig>) => {
      if (modelLocked) return;
      if (patch.model) {
        clearPendingAttachmentsForModel(patch.model);
      }
      if (activeSession) {
        try {
          const updated = await invoke<Session>("update_session", {
            req: {
              session_id: activeSession.id,
              model: patch.model,
              thinking_enabled: patch.thinking_enabled,
              thinking_effort: patch.thinking_effort,
            },
          });
          handleSessionUpdated(updated);
          const next = sessionConfigFromSession(updated);
          setPendingSessionConfig(next);
          writeStoredSessionConfig(next);
        } catch (error) {
          console.error(error);
        }
        return;
      }
      handlePendingSessionConfigChange(patch);
    },
    [activeSession, clearPendingAttachmentsForModel, handlePendingSessionConfigChange, handleSessionUpdated, modelLocked],
  );

  const reorderSessions = useCallback((activeId: string, overId: string) => {
    const projectId = activeProjectRef.current;
    if (!projectId) return;

    const next = moveSessionInList(sessionsRef.current, activeId, overId);
    if (!next) return;

    writeProjectOrder(
      projectId,
      next.map((item) => item.id),
    );
    sessionsRef.current = next;
    setSessions(next);
  }, []);

  const createSession = useCallback(async () => {
    const projectId = activeProjectRef.current;
    if (!projectId) return;

    const session = await invoke<Session>("create_session", {
      req: buildCreateSessionRequest(projectId, pendingSessionConfigRef.current),
    });
    prependSessionToOrder(projectId, session.id);
    sessionsRef.current = [session, ...sessionsRef.current];
    setSessions(sessionsRef.current);
    setActiveSessionId(session.id);
  }, []);

  const deleteSession = useCallback(
    async (sessionId: string) => {
      const projectId = activeProjectRef.current;
      if (!projectId) return;

      try {
        await invoke("delete_session", { sessionId });
        removeSessionFromOrder(projectId, sessionId);
        const next = sessionsRef.current.filter((item) => item.id !== sessionId);
        sessionsRef.current = next;
        setSessions(next);
        if (activeSessionRef.current === sessionId) {
          setActiveSessionId(mostRecentSessionId(next));
        }
      } catch (error) {
        console.error(error);
      }
    },
    [],
  );

  return {
    projects,
    setProjects,
    sessions,
    createSession,
    deleteSession,
    reorderSessions,
    messages,
    toolCalls,
    activeClarify,
    activeProjectId,
    activeSessionId,
    setActiveSessionId,
    pendingSessionConfig,
    input,
    setInput,
    pendingAttachments,
    visionToast,
    supportsVision,
    activeProjectRoot,
    addPastedImage,
    removePendingAttachment,
    dismissVisionToast,
    stream,
    contextRatio,
    apiKeyStatus,
    tavilyHasKey,
    webSearchActive,
    initializing,
    starterSuggestions,
    followupSuggestions,
    fileEntries: projectFiles.fileEntries,
    fileRevision: projectFiles.fileRevision,
    sendHint,
    highlightProject,
    highlightApiKeyProvider,
    credentialsOpen,
    setCredentialsOpen,
    credentialsHintDismissed,
    setCredentialsHintDismissed,
    modelLocked,
    models,
    effectiveSessionConfig,
    modelSummary,
    updateSessionConfig,
    sessionContextReady,
    activeProjectName,
    activity,
    showInitCapsule,
    selectProject,
    sendMessage,
    submitClarifyAnswer,
    handleInitStarter,
    handleApiKeyStatusChange,
    handleTavilyKeyChange,
    enableWebSearch,
    disableWebSearch,
    handlePendingSessionConfigChange,
    dismissSendHint,
    dismissCompactionNotice,
    handleSessionUpdated,
  };
}
