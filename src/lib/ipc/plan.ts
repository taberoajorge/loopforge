import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { PlanActivityBatchPayload, PlanSessionInfo, PlanTerminalPayload, Prd } from "./types";

export async function startPlan(args: {
  projectId: string;
  projectDir: string;
  agent: string;
  model?: string | null;
  effort?: string | null;
  initialPrompt: string;
}): Promise<void> {
  return invoke("start_plan", { args });
}

export async function writeToPlan(projectId: string, input: string): Promise<void> {
  return invoke("write_to_plan", { projectId, input });
}

export async function stopPlan(projectId: string): Promise<void> {
  return invoke("stop_plan", { projectId });
}

export async function queryPlanStatus(projectId: string): Promise<PlanSessionInfo | null> {
  return invoke<PlanSessionInfo | null>("query_plan_status", { projectId });
}

export type PlanStepState =
  | { state: "running" }
  | { state: "hasExistingPlan"; content: string }
  | { state: "needsFreshPlan" };

export interface ResolvePlanActionResult {
  action: "resume" | "prompt_existing" | "start";
  planContent?: string;
}

export type PlanUserActionKind = "feedback" | "replan";

export interface PlanUserActionResult {
  mode: "sent" | "replanned";
}

export async function resolvePlanState(projectId: string): Promise<PlanStepState> {
  return invoke<PlanStepState>("resolve_plan_state", { projectId });
}

export async function resolvePlanAction(projectId: string): Promise<ResolvePlanActionResult> {
  return invoke<ResolvePlanActionResult>("resolve_plan_action", { projectId });
}

export async function planUserAction(
  projectId: string,
  input: string,
  action: PlanUserActionKind,
): Promise<PlanUserActionResult> {
  return invoke<PlanUserActionResult>("plan_user_action", { projectId, input, action });
}

export async function loadExistingPlan(projectId: string): Promise<string | null> {
  return invoke<string | null>("load_existing_plan", { projectId });
}

export async function savePlan(projectId: string, content: string): Promise<void> {
  return invoke("save_plan", { projectId, content });
}

export async function loadExistingPrd(projectId: string): Promise<Prd | null> {
  return invoke<Prd | null>("load_existing_prd", { projectId });
}

export async function savePrd(projectId: string, prdJson: string): Promise<void> {
  return invoke("save_prd", { projectId, prdJson });
}

export async function saveConfig(projectId: string, configJson: string): Promise<void> {
  return invoke("save_config", { projectId, configJson });
}

export function onPlanActivityBatch(
  callback: (payload: PlanActivityBatchPayload) => void,
): Promise<UnlistenFn> {
  return listen<PlanActivityBatchPayload>("plan:activity-batch", (event) =>
    callback(event.payload),
  );
}

export function onPlanComplete(
  callback: (payload: PlanTerminalPayload) => void,
): Promise<UnlistenFn> {
  return listen<PlanTerminalPayload>("plan:complete", (event) => callback(event.payload));
}

export function onPlanError(callback: (payload: PlanTerminalPayload) => void): Promise<UnlistenFn> {
  return listen<PlanTerminalPayload>("plan:error", (event) => callback(event.payload));
}

export function onPlanHeartbeat(
  callback: (payload: PlanTerminalPayload) => void,
): Promise<UnlistenFn> {
  return listen<PlanTerminalPayload>("plan:heartbeat", (event) => callback(event.payload));
}
