import { ChatPanel } from "./components/ChatPanel";
import { Sidebar } from "./components/Sidebar";
import { RightPanel } from "./components/RightPanel";
import { useWorkspace } from "./hooks/useWorkspace";

function App() {
  const ws = useWorkspace();

  return (
    <div className="flex h-full flex-col bg-[#0b1020]">
      <header className="flex items-center gap-3 border-b border-slate-800 px-3 py-1.5">
        <img src="/logo.svg" alt="" className="h-5 w-5 shrink-0" aria-hidden />
        <div className="text-sm font-semibold text-white">Doc Agent</div>
        <div className="truncate text-xs text-slate-400">
          {ws.activeProjectName ? ws.activeProjectName : "请选择项目目录"}
        </div>
      </header>
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
