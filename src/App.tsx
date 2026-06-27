import { useCallback, useEffect, useMemo, useState } from "react";
import { ChatPanel } from "./components/ChatPanel";
import { CommandPalette } from "./components/CommandPalette";
import type { CommandPaletteItem } from "./lib/commandPaletteSearch";
import { parseSessionPaletteItemId } from "./lib/commandPaletteSearch";
import { CredentialsButton } from "./components/CredentialsButton";
import { CredentialsDrawer } from "./components/CredentialsDrawer";
import { CredentialsHintBanner } from "./components/CredentialsHintBanner";
import { InspectorTabs } from "./components/InspectorTabs";
import { Logo } from "./components/Logo";
import { SettingsButton } from "./components/SettingsButton";
import { SettingsDrawer } from "./components/SettingsDrawer";
import { Sidebar } from "./components/Sidebar";
import { UpdateProgressOverlay } from "./components/UpdateProgressOverlay";
import { ThemeToggle } from "./components/ThemeToggle";
import { WorkspaceLayout } from "./components/WorkspaceLayout";
import { useAppUpdater } from "./hooks/useAppUpdater";
import { useUiScale } from "./hooks/useUiScale";
import { useWorkspace } from "./hooks/useWorkspace";
import { hasAnyLlmKey } from "./lib/credentials";
import {
  isAddProjectShortcut,
  isCommandPaletteShortcut,
  isNewSessionShortcut,
  isUiScaleShortcutBlocked,
  isZoomInShortcut,
  isZoomOutShortcut,
  isZoomResetShortcut,
} from "./lib/keyboardShortcuts";

function App() {
  const ws = useWorkspace();
  const { zoomIn, zoomOut, resetScale } = useUiScale();
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [modelFlyoutOpen, setModelFlyoutOpen] = useState(false);
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false);
  useAppUpdater();

  const showCredentialsHint = useMemo(
    () => !hasAnyLlmKey(ws.apiKeyStatus) && !ws.credentialsHintDismissed,
    [ws.apiKeyStatus, ws.credentialsHintDismissed],
  );

  function openCredentialsDrawer() {
    ws.setCredentialsOpen(true);
  }

  const openCommandPalette = useCallback(() => {
    setCommandPaletteOpen(true);
  }, []);

  const handleCommandPaletteItem = useCallback(
    (item: CommandPaletteItem) => {
      switch (item.group) {
        case "actions":
          if (item.id === "action:new-session") {
            if (!ws.activeProjectId) {
              ws.promptAddProject();
              setCommandPaletteOpen(false);
              return;
            }
            void ws.createSession();
          } else if (item.id === "action:add-project") {
            void ws.addProjectFromDialog();
          }
          break;
        case "projects":
          void ws.selectProject(item.id.slice("project:".length));
          break;
        case "sessions": {
          const parsed = parseSessionPaletteItemId(item.id);
          if (!parsed) break;
          void ws.selectProject(parsed.projectId, { preferredSessionId: parsed.sessionId });
          break;
        }
        case "commands":
          ws.insertSlashCommandPrompt(item.id.slice("command:".length));
          break;
      }
    },
    [
      ws.activeProjectId,
      ws.createSession,
      ws.addProjectFromDialog,
      ws.selectProject,
      ws.insertSlashCommandPrompt,
      ws.promptAddProject,
    ],
  );

  useEffect(() => {
    if (commandPaletteOpen) {
      void ws.refreshPaletteSessions();
    }
  }, [commandPaletteOpen, ws.refreshPaletteSessions]);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (!isUiScaleShortcutBlocked(event)) {
        if (isZoomInShortcut(event)) {
          event.preventDefault();
          zoomIn();
          return;
        }
        if (isZoomOutShortcut(event)) {
          event.preventDefault();
          zoomOut();
          return;
        }
        if (isZoomResetShortcut(event)) {
          event.preventDefault();
          resetScale();
          return;
        }
      }

      if (commandPaletteOpen) {
        if (isCommandPaletteShortcut(event)) {
          event.preventDefault();
          setCommandPaletteOpen(false);
        }
        return;
      }
      if (isCommandPaletteShortcut(event)) {
        event.preventDefault();
        setCommandPaletteOpen(true);
        return;
      }
      if (isNewSessionShortcut(event)) {
        event.preventDefault();
        if (ws.activeProjectId) {
          void ws.createSession();
        } else {
          ws.promptAddProject();
        }
        return;
      }
      if (isAddProjectShortcut(event)) {
        event.preventDefault();
        void ws.addProjectFromDialog();
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [
    commandPaletteOpen,
    ws.activeProjectId,
    ws.createSession,
    ws.addProjectFromDialog,
    ws.promptAddProject,
    zoomIn,
    zoomOut,
    resetScale,
  ]);

  return (
    <div className="flex h-full flex-col bg-app">
      <header className="flex items-center gap-3 border-b border-border px-3 py-2">
        <Logo />
        <div className="text-sm font-semibold text-fg">Doc Agent</div>
        <CredentialsHintBanner
          visible={showCredentialsHint}
          onOpenCredentials={openCredentialsDrawer}
          onDismiss={() => ws.setCredentialsHintDismissed(true)}
        />
        <div className="ml-auto flex items-center gap-2">
          <CredentialsButton
            showStatusDot={!hasAnyLlmKey(ws.apiKeyStatus)}
            onClick={openCredentialsDrawer}
          />
          <SettingsButton onClick={() => setSettingsOpen(true)} />
          <ThemeToggle />
        </div>
      </header>
      <UpdateProgressOverlay />
      {commandPaletteOpen && (
        <CommandPalette
          open
          projects={ws.projects}
          sessions={ws.paletteSessions}
          onClose={() => setCommandPaletteOpen(false)}
          onSelectItem={handleCommandPaletteItem}
        />
      )}
      <SettingsDrawer
        open={settingsOpen}
        onClose={() => setSettingsOpen(false)}
        apiKeyStatus={ws.apiKeyStatus}
      />
      <CredentialsDrawer
        open={ws.credentialsOpen}
        apiKeyStatus={ws.apiKeyStatus}
        tavilyHasKey={ws.tavilyHasKey}
        highlightApiKeyProvider={ws.highlightApiKeyProvider}
        onClose={() => ws.setCredentialsOpen(false)}
        onApiKeyStatusChange={ws.handleApiKeyStatusChange}
        onTavilyStatusChange={(has) => void ws.handleTavilyKeyChange(has)}
      />
      <WorkspaceLayout
        sidebar={
          <Sidebar
            projects={ws.projects}
            sessions={ws.sessions}
            activeProjectId={ws.activeProjectId}
            activeSessionId={ws.activeSessionId}
            sessionRunStatuses={ws.sessionRunStatuses}
            highlightProject={ws.highlightProject}
            webSearchActive={ws.webSearchActive}
            onProjectsChange={ws.setProjects}
            onSelectProject={ws.selectProject}
            onSelectSession={ws.setActiveSessionId}
            onCreateSession={(projectId) => void ws.createSession(projectId)}
            onDeleteSession={(sessionId) => ws.deleteSession(sessionId)}
            onReorderSessions={ws.reorderSessions}
            onEnableWebSearch={() => void ws.enableWebSearch()}
            onDisableWebSearch={() => void ws.disableWebSearch()}
            onOpenCommandPalette={openCommandPalette}
            onPromptAddProject={ws.promptAddProject}
            onAddProject={() => ws.addProjectFromDialog()}
          />
        }
        chat={
          <ChatPanel
            sessionId={ws.activeSessionId}
            messages={ws.messages}
            toolCalls={ws.toolCalls}
            activeClarify={ws.activeClarify}
            streamingReasoning={ws.stream.streamingReasoning}
            streamingContent={ws.stream.streamingContent}
            activity={ws.activity}
            initializing={ws.initializing}
            showInitCapsule={ws.showInitCapsule}
            starterSuggestions={ws.starterSuggestions}
            followupSuggestions={ws.followupSuggestions}
            fileEntries={ws.fileEntries}
            agentsMdStatus={ws.agentsMdStatus}
            input={ws.input}
            busy={ws.stream.busy}
            contextRatio={ws.contextRatio}
            compactionNotice={ws.stream.compactionNotice}
            sendHint={ws.sendHint}
            pendingAttachments={ws.pendingAttachments}
            visionToast={ws.visionToast}
            projectId={ws.activeProjectId}
            projects={ws.projects}
            models={ws.models}
            sessionConfig={ws.effectiveSessionConfig}
            modelLocked={ws.modelLocked}
            modelSummary={ws.modelSummary}
            apiKeyStatus={ws.apiKeyStatus}
            onSelectProject={(projectId) => void ws.selectProject(projectId)}
            onSessionConfigChange={(patch) => void ws.updateSessionConfig(patch)}
            onModelFlyoutOpenChange={setModelFlyoutOpen}
            onInputChange={ws.setInput}
            composerFocusRequest={ws.composerFocusRequest}
            onConsumeComposerFocusRequest={ws.consumeComposerFocusRequest}
            onSend={ws.sendMessage}
            onPasteImage={ws.addPastedImage}
            onRemoveAttachment={ws.removePendingAttachment}
            onDismissVisionToast={ws.dismissVisionToast}
            onNotifyToast={ws.notifyToast}
            onSubmitClarify={(payload) => void ws.submitClarifyAnswer(payload)}
            onInitStarter={() => void ws.handleInitStarter()}
            onDismissSendHint={ws.dismissSendHint}
            onDismissCompactionNotice={ws.dismissCompactionNotice}
            mergeImportedPaths={ws.mergeImportedPaths}
            showSendBlocker={ws.showSendBlocker}
            ensureActiveSession={ws.ensureActiveSession}
            supportsVision={ws.supportsVision}
            onInvalidImagePick={ws.notifyInvalidImagePick}
            runStatus={
              ws.activeSessionId ? ws.sessionRunStatuses[ws.activeSessionId] ?? "idle" : "idle"
            }
            parallelAtCapacity={ws.parallelAtCapacity}
            onCancelTurn={() => void ws.cancelTurn()}
            composerFocusBlockers={{
              settingsOpen,
              credentialsOpen: ws.credentialsOpen,
              modelFlyoutOpen,
              commandPaletteOpen,
            }}
          />
        }
        right={
          <InspectorTabs
            liveTools={ws.stream.liveTools}
            turnArtifacts={ws.stream.turnArtifacts}
            projectId={ws.activeProjectId}
            fileRevision={ws.fileRevision}
            inspectorTurnNonce={ws.inspectorTurnNonce}
            activeSessionId={ws.activeSessionId}
          />
        }
      />
    </div>
  );
}

export default App;
