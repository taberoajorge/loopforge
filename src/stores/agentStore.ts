import { create } from "zustand";

interface AgentInfo {
  name: string;
  installed: boolean;
  version: string | null;
}

interface AgentState {
  agents: AgentInfo[];
  detecting: boolean;
  setAgents: (agents: AgentInfo[]) => void;
  setDetecting: (detecting: boolean) => void;
}

export const useAgentStore = create<AgentState>()((set) => ({
  agents: [],
  detecting: false,
  setAgents: (agents) => set({ agents }),
  setDetecting: (detecting) => set({ detecting }),
}));
