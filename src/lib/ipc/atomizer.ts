import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AtomizeActivityPayload,
  AtomizeArgs,
  AtomizeProgress,
  PipelineSnapshot,
  Prd,
} from "./types";

export async function runAtomizer(args: AtomizeArgs): Promise<Prd> {
  return invoke<Prd>("run_atomizer", { args });
}

export async function loadOutputLog(projectId: string): Promise<string> {
  return invoke<string>("load_output_log", { projectId });
}

export async function getAtomizerActivityLog(projectId: string): Promise<AtomizeActivityPayload[]> {
  return invoke<AtomizeActivityPayload[]>("get_atomizer_activity_log", { projectId });
}

export async function getAtomizerPipelineState(
  projectId: string,
): Promise<PipelineSnapshot | null> {
  return invoke<PipelineSnapshot | null>("get_atomizer_pipeline_state", { projectId });
}

export function onAtomizationProgress(
  callback: (payload: AtomizeProgress) => void,
): Promise<UnlistenFn> {
  return listen<AtomizeProgress>("atomization-progress", (event) => callback(event.payload));
}

export function onAtomizationActivity(
  callback: (payload: AtomizeActivityPayload) => void,
): Promise<UnlistenFn> {
  return listen<AtomizeActivityPayload>("atomization:activity", (event) => callback(event.payload));
}
