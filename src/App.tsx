import { useMemo, useState } from "react";
import { ChatPanel } from "./components/ChatPanel";
import { CredentialsButton } from "./components/CredentialsButton";
import { CredentialsDrawer } from "./components/CredentialsDrawer";
import { CredentialsHintBanner } from "./components/CredentialsHintBanner";
import { Logo } from "./components/Logo";
import { RightPanel } from "./components/RightPanel";
import { SettingsButton } from "./components/SettingsButton";
import { SettingsDrawer } from "./components/SettingsDrawer";
import { Sidebar } from "./components/Sidebar";
import { UpdateProgressOverlay } from "./components/UpdateProgressOverlay";
import { ThemeToggle } from "./components/ThemeToggle";
import { WorkspaceLayout } from "./components/WorkspaceLayout";
import { useAppUpdater } from "./hooks/useAppUpdater";
import { useWorkspace } from "./hooks/useWorkspace";
import { hasAnyLlmKey } from "./lib/credentials";

function App() {
  const ws = useWorkspace();
  const [settingsOpen, setSettingsOpen] = useState(false);
  useAppUpdater();

  const showCredentialsHint = useMemo(
    () => !hasAnyLlmKey(ws.apiKeyStatus) && !ws.credentialsHintDismissed,
    [ws.apiKeyStatus, ws.credentialsHintDismissed],
  );

  function openCredentialsDrawer() {
    ws.setCredentialsOpen(true);
  }

  return (
    <div className="flex h-full flex-col bg-app">
      <header className="flex items-center gap-3 border-b border-border px-3 py-1.5">
        <Logo />
        <div className="text-sm font-semibold text-fg">Doc Agent</div>
        <div className="hidden min-w-0 truncate text-xs text-fg-secondary sm:block">
          {ws.activeProjectName ? ws.activeProjectName : "请选择项目目录"}
        </div>
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
            models={ws.models}
            sessionConfig={ws.effectiveSessionConfig}
            modelLocked={ws.modelLocked}
            apiKeyStatus={ws.apiKeyStatus}
            highlightProject={ws.highlightProject}
            webSearchActive={ws.webSearchActive}
            modelSummary={ws.modelSummary}
            onProjectsChange={ws.setProjects}
            onSelectProject={ws.selectProject}
            onSelectSession={ws.setActiveSessionId}
            onCreateSession={() => ws.createSession()}
            onDeleteSession={(sessionId) => ws.deleteSession(sessionId)}
            onReorderSessions={ws.reorderSessions}
            onSessionConfigChange={(patch) => void ws.updateSessionConfig(patch)}
            onEnableWebSearch={() => void ws.enableWebSearch()}
            onDisableWebSearch={() => void ws.disableWebSearch()}
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
            onInputChange={ws.setInput}
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
          />
        }
        right={
          <RightPanel
            liveTools={ws.stream.liveTools}
            turnArtifacts={ws.stream.turnArtifacts}
            projectId={ws.activeProjectId}
            fileRevision={ws.fileRevision}
          />
        }
      />
    </div>
  );
}

export default App;
