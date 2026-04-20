import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { EphemeralAnswer, IterationRow, ProjectSnapshot, StartLoopArgs } from "./types";

export interface HeartbeatPayload {
  projectId: string;
  sessionId: string;
  elapsedSecs: number;
  totalSecs: number;
  context: "rate_limit_wait" | "cooldown_wait" | "health_check_wait";
}

export interface VerificationPayload {
  projectId: string;
  sessionId: string;
  storyId: string;
  attempt: number;
  maxAttempts?: number;
  errorCount?: number;
  circuitBreaker?: boolean;
}

export interface PromptPayload {
  projectId: string;
  sessionId: string;
  storyId: string;
  sizeBytes: number;
  hash: number;
  truncated: boolean;
}

export interface StorySkippedPayload {
  projectId: string;
  sessionId: string;
  storyId: string;
  reason: string;
}

export interface AgentOutputPayload {
  projectId: string;
  sessionId: string;
  line: string;
  stream: string;
}

export interface ProjectStateChangedPayload {
  projectId: string;
  snapshot?: ProjectSnapshot;
}

export type LoopEvent =
  | { type: "session_started"; payload: unknown }
  | { type: "iteration_started"; payload: unknown }
  | { type: "iteration_completed"; payload: unknown }
  | { type: "rate_limit_detected"; payload: unknown }
  | { type: "agent_switched"; payload: unknown }
  | { type: "session_ended"; payload: unknown }
  | { type: "heartbeat"; payload: HeartbeatPayload }
  | { type: "verification_started"; payload: VerificationPayload }
  | { type: "verification_failed"; payload: VerificationPayload }
  | { type: "verification_passed"; payload: VerificationPayload }
  | { type: "prompt_built"; payload: PromptPayload }
  | { type: "story_skipped"; payload: StorySkippedPayload };

export async function startLoop(args: StartLoopArgs): Promise<string> {
  return invoke<string>("start_loop", { args });
}

export async function stopLoop(projectId: string): Promise<void> {
  return invoke("stop_loop", { projectId });
}

export async function getIterationHistory(projectId: string): Promise<IterationRow[]> {
  return invoke<IterationRow[]>("get_iteration_history", { projectId });
}

export async function getActivityFeed(projectId: string): Promise<IterationRow[]> {
  return invoke<IterationRow[]>("get_activity_feed", { projectId });
}

export async function ephemeralQuery(
  projectId: string,
  question: string,
): Promise<EphemeralAnswer> {
  return invoke<EphemeralAnswer>("ephemeral_query", { projectId, question });
}

export function onAgentOutput(
  callback: (payload: AgentOutputPayload) => void,
): Promise<UnlistenFn> {
  return listen<AgentOutputPayload>("agent-output-stream", (event) => callback(event.payload));
}

export function onIterationStarted(callback: (payload: unknown) => void): Promise<UnlistenFn> {
  return listen("loop:iteration-started", (event) => callback(event.payload));
}

export function onIterationCompleted(callback: (payload: unknown) => void): Promise<UnlistenFn> {
  return listen("loop:iteration-completed", (event) => callback(event.payload));
}

export function onSessionStarted(callback: (payload: unknown) => void): Promise<UnlistenFn> {
  return listen("loop:session-started", (event) => callback(event.payload));
}

export function onSessionEnded(callback: (payload: unknown) => void): Promise<UnlistenFn> {
  return listen("loop:session-ended", (event) => callback(event.payload));
}

export function onStoriesUpdated(
  callback: (payload: { projectId: string }) => void,
): Promise<UnlistenFn> {
  return listen<{ projectId: string }>("loop:stories-updated", (event) => callback(event.payload));
}

export function onProjectStateChanged(
  callback: (payload: ProjectStateChangedPayload) => void,
): Promise<UnlistenFn> {
  return listen<ProjectStateChangedPayload>("project:state-changed", (event) =>
    callback(event.payload),
  );
}

export function onRateLimitDetected(callback: (payload: unknown) => void): Promise<UnlistenFn> {
  return listen("loop:rate-limit-detected", (event) => callback(event.payload));
}

export function onAgentSwitched(callback: (payload: unknown) => void): Promise<UnlistenFn> {
  return listen("loop:agent-switched", (event) => callback(event.payload));
}

export function onHeartbeat(callback: (payload: HeartbeatPayload) => void): Promise<UnlistenFn> {
  return listen<HeartbeatPayload>("loop:heartbeat", (event) => callback(event.payload));
}

export function onVerificationStarted(
  callback: (payload: VerificationPayload) => void,
): Promise<UnlistenFn> {
  return listen<VerificationPayload>("loop:verification-started", (event) =>
    callback(event.payload),
  );
}

export function onVerificationFailed(
  callback: (payload: VerificationPayload) => void,
): Promise<UnlistenFn> {
  return listen<VerificationPayload>("loop:verification-failed", (event) =>
    callback(event.payload),
  );
}

export function onVerificationPassed(
  callback: (payload: VerificationPayload) => void,
): Promise<UnlistenFn> {
  return listen<VerificationPayload>("loop:verification-passed", (event) =>
    callback(event.payload),
  );
}

export function onPromptBuilt(callback: (payload: PromptPayload) => void): Promise<UnlistenFn> {
  return listen<PromptPayload>("loop:prompt-built", (event) => callback(event.payload));
}

export function onStorySkipped(
  callback: (payload: StorySkippedPayload) => void,
): Promise<UnlistenFn> {
  return listen<StorySkippedPayload>("loop:story-skipped", (event) => callback(event.payload));
}
