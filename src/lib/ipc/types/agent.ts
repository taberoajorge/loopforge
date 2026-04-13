export interface AgentInfo {
  name: string;
  binary: string;
  version: string | null;
  available: boolean;
}

export interface AgentModelOption {
  id: string;
  label: string;
}

export interface AgentEffortOption {
  id: string;
  label: string;
}

export interface AgentCapabilities {
  agent: string;
  source: string;
  supportsModel: boolean;
  supportsEffort: boolean;
  models: AgentModelOption[];
  efforts: AgentEffortOption[];
  defaultModel: string | null;
  defaultEffort: string | null;
}

export interface SystemReadiness {
  agents: AgentInfo[];
  gitAvailable: boolean;
  shellAvailable: boolean;
  platform: string;
}
