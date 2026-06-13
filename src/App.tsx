import { useState } from "react";
import { ChatPanel } from "./components/ChatPanel";
import { Logo } from "./components/Logo";
import { RightPanel } from "./components/RightPanel";
import { SettingsButton } from "./components/SettingsButton";
import { SettingsDrawer } from "./components/SettingsDrawer";
import { Sidebar } from "./components/Sidebar";
import { ThemeToggle } from "./components/ThemeToggle";
import { useAppUpdater } from "./hooks/useAppUpdater";
import { useWorkspace } from "./hooks/useWorkspace";

function App() {
  const ws = useWorkspace();
  const [settingsOpen, setSettingsOpen] = useState(false);
  useAppUpdater();

  return (
    <div className="flex h-full flex-col bg-app">
      <header className="flex items-center gap-3 border-b border-border px-3 py-1.5">
        <Logo />
        <div className="text-sm font-semibold text-fg">Doc Agent</div>
        <div className="truncate text-xs text-fg-secondary">
          {ws.activeProjectName ? ws.activeProjectName : "请选择项目目录"}
        </div>
        <div className="ml-auto flex items-center gap-2">
          <SettingsButton onClick={() => setSettingsOpen(true)} />
          <ThemeToggle />
        </div>
      </header>
      <SettingsDrawer open={settingsOpen} onClose={() => setSettingsOpen(false)} />
      <main className="flex min-h-0 flex-1 gap-2.5 p-2.5">
        <Sidebar
          projects={ws.projects}
          sessions={ws.sessions}
          activeProjectId={ws.activeProjectId}
          activeSessionId={ws.activeSessionId}
          apiKeyStatus={ws.apiKeyStatus}
          pendingSessionConfig={ws.pendingSessionConfig}
          modelLocked={ws.modelLocked}
          highlightProject={ws.highlightProject}
          highlightApiKeyProvider={ws.highlightApiKeyProvider}
          onProjectsChange={ws.setProjects}
          onSessionsChange={ws.setSessions}
          onSelectProject={ws.selectProject}
          onSelectSession={ws.setActiveSessionId}
          onPendingSessionConfigChange={ws.handlePendingSessionConfigChange}
          onSessionUpdated={ws.handleSessionUpdated}
          onApiKeyStatusChange={ws.handleApiKeyStatusChange}
          tavilyEnabled={ws.tavilyEnabled}
          onTavilyStatusChange={ws.handleTavilyStatusChange}
        />
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
          filePaths={ws.filePaths}
          input={ws.input}
          busy={ws.stream.busy}
          sendHint={ws.sendHint}
          onInputChange={ws.setInput}
          onSend={ws.sendMessage}
          onSubmitClarify={(payload) => void ws.submitClarifyAnswer(payload)}
          onInitStarter={() => void ws.handleInitStarter()}
          onDismissSendHint={ws.dismissSendHint}
        />
        <RightPanel
          liveTools={ws.stream.liveTools}
          projectId={ws.activeProjectId}
          fileRevision={ws.fileRevision}
        />
      </main>
    </div>
  );
}

export default App;
