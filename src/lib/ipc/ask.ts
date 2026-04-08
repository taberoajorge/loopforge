import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AskCompletePayload, AskErrorPayload, AskMessage, AskStreamPayload,
} from "./types";

export async function askQuestion(
  projectId: string, question: string, agent: string, model?: string | null,
): Promise<string> {
  return invoke<string>("ask_question", { args: { projectId, question, agent, model } });
}

export async function askHistory(projectId: string): Promise<AskMessage[]> {
  return invoke<AskMessage[]>("ask_history", { projectId });
}

export async function stopAsk(projectId: string): Promise<void> {
  return invoke("stop_ask", { projectId });
}

export async function copyAskMessage(messageId: string): Promise<string> {
  return invoke<string>("copy_ask_message", { messageId });
}

export async function truncateAskFrom(
  projectId: string, messageId: string,
): Promise<AskMessage[]> {
  return invoke<AskMessage[]>("truncate_ask_from", { projectId, messageId });
}

export async function retryAsk(
  projectId: string, messageId: string, agent: string, model?: string | null,
): Promise<string> {
  return invoke<string>("retry_ask", { projectId, messageId, agent, model });
}

export function onAskStream(
  callback: (payload: AskStreamPayload) => void,
): Promise<UnlistenFn> {
  return listen<AskStreamPayload>("ask:stream", (event) => callback(event.payload));
}

export function onAskComplete(
  callback: (payload: AskCompletePayload) => void,
): Promise<UnlistenFn> {
  return listen<AskCompletePayload>("ask:complete", (event) => callback(event.payload));
}

export function onAskError(
  callback: (payload: AskErrorPayload) => void,
): Promise<UnlistenFn> {
  return listen<AskErrorPayload>("ask:error", (event) => callback(event.payload));
}
