import { invoke } from "@tauri-apps/api/core";
import type { AgentCapabilities, AgentInfo } from "./types";

export async function detectAgents(): Promise<AgentInfo[]> {
  return invoke<AgentInfo[]>("detect_agents");
}

export async function refreshAgents(): Promise<AgentInfo[]> {
  return invoke<AgentInfo[]>("refresh_agents");
}

export async function getAgentCapabilities(agent: string): Promise<AgentCapabilities> {
  return invoke<AgentCapabilities>("get_agent_capabilities", { agent });
}
