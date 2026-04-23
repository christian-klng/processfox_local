import { invoke } from "@tauri-apps/api/core";

import type { Agent, AgentDraft, AgentUpdate } from "@/types/agent";
import type { FileEntry } from "@/types/file";

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
