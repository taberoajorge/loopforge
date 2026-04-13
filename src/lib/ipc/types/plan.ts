import type { PlanEventKind } from "../../../types/wizard";

export interface PlanActivityPayload {
  projectId: string;
  kind: PlanEventKind;
  content: string;
  timestamp: string;
}

export interface PlanTerminalPayload {
  projectId: string;
  detail: string;
  finalContent?: string;
}

export interface PlanActivityBatchPayload {
  projectId: string;
  events: PlanActivityPayload[];
  planContent: string;
  planContentDelta: string;
}

export type PlanSessionStatus = "running" | "complete" | "failed" | "stalled";

export interface PlanSessionInfo {
  status: PlanSessionStatus;
  agentName: string;
  elapsedSecs: number;
  lastActivitySecsAgo: number;
}
