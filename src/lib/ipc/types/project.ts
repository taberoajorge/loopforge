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
  progressPercent: number;
  uptimeLabel: string;
}

export interface IterationStory {
  id: string;
  title: string;
  status: "completed" | "current" | "blocked" | "pending";
  durationSecs?: number;
  durationLabel?: string;
  attempts: number;
}

export interface WizardResumeState {
  project: ProjectRecord;
  wizardStep: string;
  wizardStateJson: string | null;
  hasPlan: boolean;
  hasPrd: boolean;
}
