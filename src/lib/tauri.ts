import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { PlanEventKind, UserStory } from "../stores/wizardStore";

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

export interface StartLoopArgs {
  projectId: string;
  agent?: string;
  model?: string | null;
  effort?: string | null;
}

export interface AtomizeArgs {
  projectId: string;
  projectName: string;
  projectDir: string;
  agent: string;
  model?: string | null;
  effort?: string | null;
}

export interface AtomizeProgress {
  stage: number;
  stageName: string;
  message: string;
  projectId: string;
}

export interface Prd {
  projectName: string;
  generatedAt: string;
  totalEstimatedMinutes: number;
  stories: UserStory[];
}

export interface ProjectRecord {
  id: string;
  name: string;
  description: string;
  status: string;
  workingDirectory: string;
  createdAt: string;
  updatedAt: string;
}

export interface ProjectsByStatus {
  active: ProjectRecord[];
  paused: ProjectRecord[];
  completed: ProjectRecord[];
  draft: ProjectRecord[];
  archived: ProjectRecord[];
  blocked: ProjectRecord[];
  failed: ProjectRecord[];
}

export type SnapshotStatus =
  | "draft"
  | "ready"
  | "running"
  | "paused"
  | "blocked"
  | "failed"
  | "completed"
  | "archived";

export interface ArtifactPaths {
  root: string;
  draft: string;
  plan: string;
  prd: string;
  config: string;
  prompt: string;
  guardrails: string;
}

export interface SessionInfo {
  id: string;
  startedAt: string | null;
  endedAt: string | null;
}

export interface ProgressInfo {
  storiesTotal: number;
  storiesDone: number;
  currentStory: string | null;
}

export interface ProjectSnapshot {
  project: ProjectRecord & { wizardStep?: string | null };
  status: SnapshotStatus;
  activeSession: SessionInfo | null;
  progress: ProgressInfo;
  config: ProjectConfig | null;
  artifactPaths: ArtifactPaths;
}

export interface IterationStory {
  id: string;
  title: string;
  status: "completed" | "current" | "blocked" | "pending";
  durationSecs?: number;
  attempts: number;
}

export async function listProjects(): Promise<ProjectsByStatus> {
  return invoke<ProjectsByStatus>("list_projects");
}

export async function createProject(name: string, description: string, workingDirectory: string, wizardStep?: string): Promise<ProjectRecord> {
  return invoke<ProjectRecord>("create_project", { name, description, workingDirectory, wizardStep });
}

export async function detectAgents(): Promise<AgentInfo[]> {
  return invoke<AgentInfo[]>("detect_agents");
}

export async function refreshAgents(): Promise<AgentInfo[]> {
  return invoke<AgentInfo[]>("refresh_agents");
}

export async function getAgentCapabilities(agent: string): Promise<AgentCapabilities> {
  return invoke<AgentCapabilities>("get_agent_capabilities", { agent });
}

export async function getProjectSnapshot(projectId: string): Promise<ProjectSnapshot> {
  return invoke<ProjectSnapshot>("get_project_snapshot", { projectId });
}

export async function saveWizardState(projectId: string, wizardStep: string, wizardStateJson: string): Promise<void> {
  return invoke("save_wizard_state", { projectId, wizardStep, wizardStateJson });
}

export async function saveDraft(projectId: string, draftJson: string): Promise<void> {
  return invoke("save_draft", { projectId, draftJson });
}

export async function loadDraft(projectId: string): Promise<string | null> {
  return invoke<string | null>("load_draft", { projectId });
}

export interface WizardResumeState {
  project: ProjectRecord;
  wizardStep: string;
  wizardStateJson: string | null;
  hasPlan: boolean;
  hasPrd: boolean;
}

export async function resumeWizard(projectId: string): Promise<WizardResumeState> {
  return invoke<WizardResumeState>("resume_wizard", { projectId });
}

export async function startLoop(args: StartLoopArgs): Promise<string> {
  return invoke<string>("start_loop", { args });
}

export async function stopLoop(projectId: string): Promise<void> {
  return invoke("stop_loop", { projectId });
}

export async function pauseProject(projectId: string): Promise<void> {
  return invoke("pause_project", { projectId });
}

export async function resumeProject(projectId: string): Promise<void> {
  return invoke("resume_project", { projectId });
}

export async function finalizeDraft(projectId: string): Promise<void> {
  return invoke("finalize_draft", { projectId });
}

export async function discardDraft(projectId: string): Promise<void> {
  return invoke("discard_draft", { projectId });
}

export async function getProjectStories(projectId: string): Promise<IterationStory[]> {
  return invoke<IterationStory[]>("get_project_stories", { projectId });
}

export interface ProjectConfig {
  schemaVersion: number;
  executeAgent: string;
  executeModel?: string | null;
  executeEffort?: string | null;
  fallbackChain: string[];
  gutterThreshold: number;
  maxIterations: number;
  cooldownSeconds: number;
  testCommand: string;
  maxVerificationRetries: number;
  scmProvider: string;
  reviewPollingInterval: number;
  reviewTimeout: number;
}

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

export interface PlanActivityPayload {
  projectId: string;
  kind: PlanEventKind;
  content: string;
  timestamp: string;
}

export interface PlanTerminalPayload {
  projectId: string;
  detail: string;
}

export interface PlanActivityBatchPayload {
  projectId: string;
  events: PlanActivityPayload[];
  planContentDelta: string;
}

export function onPlanActivityBatch(callback: (payload: PlanActivityBatchPayload) => void): Promise<UnlistenFn> {
  return listen<PlanActivityBatchPayload>("plan:activity-batch", (event) => callback(event.payload));
}

export function onPlanComplete(callback: (payload: PlanTerminalPayload) => void): Promise<UnlistenFn> {
  return listen<PlanTerminalPayload>("plan:complete", (event) => callback(event.payload));
}

export function onPlanError(callback: (payload: PlanTerminalPayload) => void): Promise<UnlistenFn> {
  return listen<PlanTerminalPayload>("plan:error", (event) => callback(event.payload));
}

export function onPlanHeartbeat(callback: (payload: PlanTerminalPayload) => void): Promise<UnlistenFn> {
  return listen<PlanTerminalPayload>("plan:heartbeat", (event) => callback(event.payload));
}

export async function writeToPlan(projectId: string, input: string): Promise<void> {
  return invoke("write_to_plan", { projectId, input });
}

export async function stopPlan(projectId: string): Promise<void> {
  return invoke("stop_plan", { projectId });
}

export type PlanSessionStatus = "running" | "complete" | "failed" | "stalled";

export interface PlanSessionInfo {
  status: PlanSessionStatus;
  agentName: string;
  elapsedSecs: number;
  lastActivitySecsAgo: number;
}

export async function queryPlanStatus(projectId: string): Promise<PlanSessionInfo | null> {
  return invoke<PlanSessionInfo | null>("query_plan_status", { projectId });
}

export async function loadExistingPlan(projectId: string): Promise<string | null> {
  return invoke<string | null>("load_existing_plan", { projectId });
}

export async function runAtomizer(args: AtomizeArgs): Promise<Prd> {
  return invoke<Prd>("run_atomizer", { args });
}

export async function loadExistingPrd(projectId: string): Promise<Prd | null> {
  return invoke<Prd | null>("load_existing_prd", { projectId });
}

export async function loadOutputLog(projectId: string): Promise<string> {
  return invoke<string>("load_output_log", { projectId });
}

export async function savePlan(projectId: string, content: string): Promise<void> {
  return invoke("save_plan", { projectId, content });
}

export async function savePrd(projectId: string, prdJson: string): Promise<void> {
  return invoke("save_prd", { projectId, prdJson });
}

export async function saveConfig(projectId: string, configJson: string): Promise<void> {
  return invoke("save_config", { projectId, configJson });
}

export interface IterationRow {
  storyId: string;
  startedAt: string;
  durationSecs: number;
  result: string;
  agentUsed: string;
}

export async function getIterationHistory(projectId: string): Promise<IterationRow[]> {
  return invoke<IterationRow[]>("get_iteration_history", { projectId });
}

export interface EphemeralAnswer {
  question: string;
  answer: string;
  source: "instant" | "agent";
}

export async function ephemeralQuery(projectId: string, question: string): Promise<EphemeralAnswer> {
  return invoke<EphemeralAnswer>("ephemeral_query", { projectId, question });
}

export interface ConnectionRepo {
  repoPath: string;
  displayName: string | null;
}

export interface Connection {
  id: string;
  name: string;
  createdAt: string;
  repos: ConnectionRepo[];
}

export async function listConnections(): Promise<Connection[]> {
  return invoke<Connection[]>("list_connections");
}

export async function buildConnectionWorkspace(connectionId: string): Promise<string> {
  return invoke<string>("build_connection_workspace", { connectionId });
}


export interface AskMessage {
  id: string;
  conversationId: string;
  role: "user" | "assistant";
  content: string;
  agent: string | null;
  model: string | null;
  createdAt: string;
}

export interface AskStreamPayload {
  projectId: string;
  messageId: string;
  chunk: string;
}

export interface AskCompletePayload {
  projectId: string;
  messageId: string;
  fullContent: string;
  agent: string;
  model: string | null;
}

export interface AskErrorPayload {
  projectId: string;
  messageId: string;
  error: string;
}

export async function askQuestion(projectId: string, question: string, agent: string, model?: string | null): Promise<string> {
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

export async function truncateAskFrom(projectId: string, messageId: string): Promise<AskMessage[]> {
  return invoke<AskMessage[]>("truncate_ask_from", { projectId, messageId });
}

export async function retryAsk(projectId: string, messageId: string, agent: string, model?: string | null): Promise<string> {
  return invoke<string>("retry_ask", { projectId, messageId, agent, model });
}

export function onAskStream(callback: (payload: AskStreamPayload) => void): Promise<UnlistenFn> {
  return listen<AskStreamPayload>("ask:stream", (event) => callback(event.payload));
}

export function onAskComplete(callback: (payload: AskCompletePayload) => void): Promise<UnlistenFn> {
  return listen<AskCompletePayload>("ask:complete", (event) => callback(event.payload));
}

export function onAskError(callback: (payload: AskErrorPayload) => void): Promise<UnlistenFn> {
  return listen<AskErrorPayload>("ask:error", (event) => callback(event.payload));
}

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

export interface AgentOutputPayload {
  projectId: string;
  sessionId: string;
  line: string;
  stream: string;
}

export function onAgentOutput(callback: (payload: AgentOutputPayload) => void): Promise<UnlistenFn> {
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

export function onRateLimitDetected(callback: (payload: unknown) => void): Promise<UnlistenFn> {
  return listen("loop:rate-limit-detected", (event) => callback(event.payload));
}

export function onAgentSwitched(callback: (payload: unknown) => void): Promise<UnlistenFn> {
  return listen("loop:agent-switched", (event) => callback(event.payload));
}

export function onAtomizationProgress(callback: (payload: AtomizeProgress) => void): Promise<UnlistenFn> {
  return listen<AtomizeProgress>("atomization-progress", (event) => callback(event.payload));
}

export function onHeartbeat(callback: (payload: HeartbeatPayload) => void): Promise<UnlistenFn> {
  return listen<HeartbeatPayload>("loop:heartbeat", (event) => callback(event.payload));
}

export function onVerificationStarted(callback: (payload: VerificationPayload) => void): Promise<UnlistenFn> {
  return listen<VerificationPayload>("loop:verification-started", (event) => callback(event.payload));
}

export function onVerificationFailed(callback: (payload: VerificationPayload) => void): Promise<UnlistenFn> {
  return listen<VerificationPayload>("loop:verification-failed", (event) => callback(event.payload));
}

export function onVerificationPassed(callback: (payload: VerificationPayload) => void): Promise<UnlistenFn> {
  return listen<VerificationPayload>("loop:verification-passed", (event) => callback(event.payload));
}

export function onPromptBuilt(callback: (payload: PromptPayload) => void): Promise<UnlistenFn> {
  return listen<PromptPayload>("loop:prompt-built", (event) => callback(event.payload));
}

export function onStorySkipped(callback: (payload: StorySkippedPayload) => void): Promise<UnlistenFn> {
  return listen<StorySkippedPayload>("loop:story-skipped", (event) => callback(event.payload));
}
