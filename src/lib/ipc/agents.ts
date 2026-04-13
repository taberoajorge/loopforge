import { invoke } from "@tauri-apps/api/core";
import type { AgentCapabilities, AgentInfo, SystemReadiness } from "./types";

export interface ResolvedAgentSelection {
  capabilities: AgentCapabilities;
  resolvedModel: string | null;
  resolvedEffort: string | null;
}

export async function detectAgents(): Promise<AgentInfo[]> {
  return invoke<AgentInfo[]>("detect_agents");
}

export async function refreshAgents(): Promise<AgentInfo[]> {
  return invoke<AgentInfo[]>("refresh_agents");
}

export async function checkSystemReadiness(): Promise<SystemReadiness> {
  return invoke<SystemReadiness>("check_system_readiness");
}

export async function getKnownAgents(): Promise<string[]> {
  return invoke<string[]>("get_known_agents");
}

export async function getAgentCapabilities(agent: string): Promise<AgentCapabilities> {
  return invoke<AgentCapabilities>("get_agent_capabilities", { agent });
}

export async function resolveAgentSelection(
  agent: string,
  currentModel: string | null,
  currentEffort: string | null,
): Promise<ResolvedAgentSelection> {
  return invoke<ResolvedAgentSelection>("resolve_agent_selection", {
    agent,
    currentModel,
    currentEffort,
  });
}
