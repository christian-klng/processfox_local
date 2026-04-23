import { useCallback, useEffect, useState } from "react";

import { AgentEditorDialog } from "@/components/agent/AgentEditorDialog";
import { ThemeProvider } from "@/components/theme-provider";
import { Main } from "@/views/Main";
import { SettingsDialog } from "@/views/Settings";
import { agentApi } from "@/lib/tauri";
import type { Agent } from "@/types/agent";
import type { Message } from "@/types/message";

type SelectedFile = { path: string; name: string } | null;

export default function App() {
  return (
    <ThemeProvider>
      <AppShell />
    </ThemeProvider>
  );
}

function AppShell() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [activeAgent, setActiveAgent] = useState<Agent | null>(null);
  const [messagesByAgent, setMessagesByAgent] = useState<Record<string, Message[]>>({});
  const [selectedFile, setSelectedFile] = useState<SelectedFile>(null);

  const [settingsOpen, setSettingsOpen] = useState(false);
  const [agentEditor, setAgentEditor] = useState<
    { mode: "create" | "edit" } | null
  >(null);

  const refreshAgents = useCallback(async () => {
    const list = await agentApi.list();
    setAgents(list);
    return list;
  }, []);

  useEffect(() => {
    refreshAgents()
      .then((list) => {
        if (list.length > 0 && !activeAgent) setActiveAgent(list[0]);
      })
      .catch((e) => console.error("failed to load agents", e));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleSelectAgent = useCallback(
    (agent: Agent) => {
      setActiveAgent(agent);
      setSelectedFile(null); // per Phase-1-Entscheidung: Reload nur bei Agent-Wechsel
    },
    [],
  );

  const handleCreateAgent = useCallback(() => {
    setAgentEditor({ mode: "create" });
  }, []);

  const handleEditAgent = useCallback(() => {
    if (!activeAgent) return;
    setAgentEditor({ mode: "edit" });
  }, [activeAgent]);

  const handleAgentSaved = useCallback(
    async (saved: Agent) => {
      await refreshAgents();
      setActiveAgent(saved);
      setSelectedFile(null);
    },
    [refreshAgents],
  );

  const handleSelectFile = useCallback((path: string, name: string) => {
    setSelectedFile({ path, name });
  }, []);

  const handleClosePreview = useCallback(() => setSelectedFile(null), []);

  const handleSendMessage = useCallback(
    (text: string) => {
      if (!activeAgent) return;
      const msg: Message = {
        id: crypto.randomUUID(),
        role: "user",
        content: text,
        createdAt: new Date().toISOString(),
      };
      setMessagesByAgent((prev) => ({
        ...prev,
        [activeAgent.id]: [...(prev[activeAgent.id] ?? []), msg],
      }));
    },
    [activeAgent],
  );

  const messages = activeAgent
    ? (messagesByAgent[activeAgent.id] ?? [])
    : [];

  return (
    <div className="flex h-full w-full flex-col">
      <Main
        agents={agents}
        activeAgent={activeAgent}
        selectedFile={selectedFile}
        messages={messages}
        onSelectAgent={handleSelectAgent}
        onCreateAgent={handleCreateAgent}
        onEditAgent={handleEditAgent}
        onOpenSettings={() => setSettingsOpen(true)}
        onSelectFile={handleSelectFile}
        onClosePreview={handleClosePreview}
        onSendMessage={handleSendMessage}
      />

      <AgentEditorDialog
        open={agentEditor !== null}
        mode={agentEditor?.mode ?? "create"}
        agent={agentEditor?.mode === "edit" ? activeAgent : null}
        onClose={() => setAgentEditor(null)}
        onSaved={handleAgentSaved}
      />

      <SettingsDialog
        open={settingsOpen}
        onClose={() => setSettingsOpen(false)}
      />
    </div>
  );
}
