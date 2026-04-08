import type { PlanEventKind, UserStory } from "../../types/wizard";

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

export type SnapshotStatus =
  | "draft" | "ready" | "running" | "paused"
  | "blocked" | "failed" | "completed" | "archived";

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

export interface WizardResumeState {
  project: ProjectRecord;
  wizardStep: string;
  wizardStateJson: string | null;
  hasPlan: boolean;
  hasPrd: boolean;
}

export interface Prd {
  projectName: string;
  generatedAt: string;
  totalEstimatedMinutes: number;
  stories: UserStory[];
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

export interface IterationRow {
  storyId: string;
  startedAt: string;
  durationSecs: number;
  result: string;
  agentUsed: string;
}

export interface EphemeralAnswer {
  question: string;
  answer: string;
  source: "instant" | "agent";
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

export type PlanSessionStatus = "running" | "complete" | "failed" | "stalled";

export interface PlanSessionInfo {
  status: PlanSessionStatus;
  agentName: string;
  elapsedSecs: number;
  lastActivitySecsAgo: number;
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
