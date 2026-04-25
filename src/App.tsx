import { useCallback, useEffect, useMemo, useState } from "react";

import { AgentEditorDialog } from "@/components/agent/AgentEditorDialog";
import { ThemeProvider } from "@/components/theme-provider";
import { resolveAgentModel, useAgentChat } from "@/hooks/useAgentChat";
import { Main } from "@/views/Main";
import { SettingsDialog } from "@/views/Settings";
import { agentApi, modelsApi, secretsApi, settingsApi, skillsApi } from "@/lib/tauri";
import type { Agent } from "@/types/agent";
import type { InstalledModel } from "@/types/models";
import type { Settings } from "@/types/settings";
import type { Skill } from "@/types/skill";

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
  const [settings, setSettings] = useState<Settings | null>(null);
  const [hasApiKey, setHasApiKey] = useState<boolean | null>(null);
  const [installedModels, setInstalledModels] = useState<InstalledModel[]>([]);
  const [skills, setSkills] = useState<Skill[]>([]);
  const [selectedFile, setSelectedFile] = useState<SelectedFile>(null);

  const [settingsState, setSettingsState] = useState<
    { open: false } | { open: true; tab: "models" | "cloud" | "appearance" | "about" }
  >({ open: false });
  const [agentEditor, setAgentEditor] = useState<
    { mode: "create" | "edit" } | null
  >(null);

  const effectiveModel = useMemo(
    () => resolveAgentModel(activeAgent, settings),
    [activeAgent, settings],
  );

  const chat = useAgentChat(activeAgent, effectiveModel);

  const refreshAgents = useCallback(async () => {
    const list = await agentApi.list();
    setAgents(list);
    return list;
  }, []);

  const refreshSettings = useCallback(async () => {
    const s = await settingsApi.get();
    setSettings(s);
    return s;
  }, []);

  useEffect(() => {
    skillsApi.list().then(setSkills).catch(console.error);
  }, []);

  useEffect(() => {
    Promise.all([refreshAgents(), refreshSettings()])
      .then(([list, s]) => {
        if (list.length > 0) setActiveAgent(list[0]);
        // First-run onboarding: open Settings on the Models tab so the user
        // can either download a local model or configure a cloud provider.
        if (!s.firstRunDone) {
          setSettingsState({ open: true, tab: "models" });
        }
      })
      .catch((e) => console.error("initial load failed", e));
  }, [refreshAgents, refreshSettings]);

  // Refresh the installed-models list whenever Settings closes (the user may
  // have downloaded or deleted a model). This feeds the local-model gate below.
  useEffect(() => {
    if (settingsState.open) return;
    modelsApi.listInstalled().then(setInstalledModels).catch(console.error);
  }, [settingsState.open]);

  // Check API key status for the effective provider. Local models don't use
  // keychain credentials, so we short-circuit for them.
  useEffect(() => {
    if (!effectiveModel) {
      setHasApiKey(null);
      return;
    }
    if (effectiveModel.provider === "local") {
      setHasApiKey(null);
      return;
    }
    let cancelled = false;
    secretsApi
      .hasApiKey(effectiveModel.provider)
      .then((ok) => {
        if (!cancelled) setHasApiKey(ok);
      })
      .catch(() => {
        if (!cancelled) setHasApiKey(false);
      });
    return () => {
      cancelled = true;
    };
  }, [effectiveModel, settingsState.open]);

  const handleSelectAgent = useCallback((agent: Agent) => {
    setActiveAgent(agent);
    setSelectedFile(null);
  }, []);

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

  const handleOpenSettings = useCallback(
    () => setSettingsState({ open: true, tab: "cloud" }),
    [],
  );
  const handleCloseSettings = useCallback(() => {
    setSettingsState({ open: false });
    // Re-fetch settings on close in case the user changed a default.
    refreshSettings().catch(console.error);
  }, [refreshSettings]);

  const handleSettingsChange = useCallback((s: Settings) => {
    setSettings(s);
  }, []);

  // Compute chat disabled state and reason.
  const { chatDisabled, chatDisabledReason } = (() => {
    if (!activeAgent) {
      return {
        chatDisabled: true,
        chatDisabledReason: "Leg zunächst einen Agenten an." as string | undefined,
      };
    }
    if (!effectiveModel) {
      return {
        chatDisabled: true,
        chatDisabledReason:
          "Kein Modell konfiguriert — in den Einstellungen einen Default setzen oder im Agenten überschreiben.",
      };
    }
    if (effectiveModel.provider === "local") {
      const present = installedModels.some(
        (m) => m.filename === effectiveModel.modelId,
      );
      if (!present) {
        return {
          chatDisabled: true,
          chatDisabledReason: `Lokales Modell „${effectiveModel.modelId}" ist nicht installiert.`,
        };
      }
    } else if (hasApiKey === false) {
      return {
        chatDisabled: true,
        chatDisabledReason: `Kein API-Key für ${effectiveModel.provider} hinterlegt.`,
      };
    }
    return {
      chatDisabled: false,
      chatDisabledReason: undefined as string | undefined,
    };
  })();

  return (
    <div className="flex h-full w-full flex-col">
      <Main
        agents={agents}
        activeAgent={activeAgent}
        selectedFile={selectedFile}
        messages={chat.messages}
        streamingText={chat.streamingText}
        pendingTools={chat.pendingTools}
        sending={chat.sending}
        chatError={chat.error}
        chatDisabled={chatDisabled}
        chatDisabledReason={chatDisabledReason}
        skills={skills}
        onSelectAgent={handleSelectAgent}
        onCreateAgent={handleCreateAgent}
        onEditAgent={handleEditAgent}
        onOpenSettings={handleOpenSettings}
        onSelectFile={handleSelectFile}
        onClosePreview={handleClosePreview}
        onSendMessage={chat.send}
        onCancelRun={chat.cancel}
        onDismissChatError={chat.clearError}
      />

      <AgentEditorDialog
        open={agentEditor !== null}
        mode={agentEditor?.mode ?? "create"}
        agent={agentEditor?.mode === "edit" ? activeAgent : null}
        onClose={() => setAgentEditor(null)}
        onSaved={handleAgentSaved}
      />

      <SettingsDialog
        open={settingsState.open}
        defaultTab={settingsState.open ? settingsState.tab : undefined}
        onClose={handleCloseSettings}
        onSettingsChange={handleSettingsChange}
      />
    </div>
  );
}
