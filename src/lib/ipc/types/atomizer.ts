import type { AtomizeActivityKind, UserStory } from "../../../types/wizard";

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
  elapsedMs: number;
}

export interface AtomizeActivityPayload {
  projectId: string;
  kind: AtomizeActivityKind;
  content: string;
  timestamp: string;
}

export type StageStatus = "pending" | "running" | "done" | "error";

export interface StageSnapshot {
  number: number;
  label: string;
  status: StageStatus;
}

export interface PipelineSnapshot {
  stages: StageSnapshot[];
  error: string | null;
  startedAt: string | null;
  elapsedMs: number;
  done: boolean;
  percent: number;
  isDone: boolean;
  isRunning: boolean;
}

export interface IterationRow {
  storyId: string;
  startedAt: string;
  durationSecs: number;
  durationLabel: string;
  timeLabel: string;
  result: string;
  agentUsed: string;
}

export interface EphemeralAnswer {
  question: string;
  answer: string;
  source: "instant" | "agent";
}
