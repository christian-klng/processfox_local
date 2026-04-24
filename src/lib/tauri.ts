import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type { Agent, AgentDraft, AgentUpdate } from "@/types/agent";
import type { ChatMessage, RunEvent, RunStarted } from "@/types/chat";
import type { FileEntry } from "@/types/file";
import type { Settings } from "@/types/settings";

export const agentApi = {
  list: () => invoke<Agent[]>("list_agents"),
  get: (id: string) => invoke<Agent>("get_agent", { id }),
  create: (draft: AgentDraft) => invoke<Agent>("create_agent", { draft }),
  update: (id: string, update: AgentUpdate) =>
    invoke<Agent>("update_agent", { id, update }),
  delete: (id: string) => invoke<void>("delete_agent", { id }),
};

export const fileApi = {
  listAgentFolder: (agentId: string, subPath?: string) =>
    invoke<FileEntry[]>("list_agent_folder", { agentId, subPath }),
};

export const settingsApi = {
  get: () => invoke<Settings>("get_settings"),
  setDefaultProvider: (provider: string | null) =>
    invoke<Settings>("set_default_provider", { provider }),
  setDefaultModel: (provider: string, model: string | null) =>
    invoke<Settings>("set_default_model", { provider, model }),
  setFirstRunDone: () => invoke<Settings>("set_first_run_done"),
  availableProviders: () => invoke<string[]>("available_providers"),
};

export interface ValidationResult {
  ok: boolean;
  error?: string;
}

export const secretsApi = {
  setApiKey: (provider: string, value: string) =>
    invoke<void>("set_api_key", { provider, value }),
  hasApiKey: (provider: string) =>
    invoke<boolean>("has_api_key", { provider }),
  clearApiKey: (provider: string) =>
    invoke<void>("clear_api_key", { provider }),
  validateApiKey: (provider: string) =>
    invoke<ValidationResult>("validate_api_key", { provider }),
};

export const chatApi = {
  listMessages: (agentId: string) =>
    invoke<ChatMessage[]>("list_messages", { agentId }),

  sendMessage: (params: {
    agentId: string;
    provider: string;
    modelId: string;
    text: string;
  }) => invoke<RunStarted>("send_message", params),

  cancelRun: (runId: string) => invoke<void>("cancel_run", { runId }),

  subscribeRun: (
    runId: string,
    handler: (event: RunEvent) => void,
  ): Promise<UnlistenFn> =>
    listen<RunEvent>(`chat:run:${runId}`, (evt) => handler(evt.payload)),
};
