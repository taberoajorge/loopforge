import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { AtomizeArgs, AtomizeProgress, Prd } from "./types";

export async function runAtomizer(args: AtomizeArgs): Promise<Prd> {
  return invoke<Prd>("run_atomizer", { args });
}

export async function loadOutputLog(projectId: string): Promise<string> {
  return invoke<string>("load_output_log", { projectId });
}

export function onAtomizationProgress(
  callback: (payload: AtomizeProgress) => void,
): Promise<UnlistenFn> {
  return listen<AtomizeProgress>("atomization-progress", (event) => callback(event.payload));
}
