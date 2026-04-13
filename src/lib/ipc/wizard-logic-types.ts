import type { UserStory } from "../../types/wizard";
import type { ProjectConfig } from "./types";

export interface AdvanceWizardResult {
  currentStep: number;
  highestStep: number;
  stepName: string;
}

export interface ConfigLimits {
  gutterThreshold: [number, number];
  maxIterations: [number, number];
  cooldownSeconds: [number, number];
  maxVerificationRetries: [number, number];
  reviewPollingInterval: [number, number];
  reviewTimeout: [number, number];
}

export interface ConfigDefaultsResponse {
  config: ProjectConfig;
  limits: ConfigLimits;
}

export interface ValidationErrors {
  errors: Record<string, string>;
}

export interface LaunchReadiness {
  ready: boolean;
  issues: string[];
  totalEstimatedMinutes: number;
  totalEstimatedHours: number;
}

export interface StoriesResponse {
  stories: UserStory[];
  totalEstimatedMinutes: number;
}

export interface WizardProjectData {
  name: string;
  description: string;
  workingDirectory: string;
  planAgent: string;
  planModel: string | null;
  planEffort: string | null;
}

export interface WizardHydrationResult {
  project: {
    id: string;
    name: string;
    description: string;
    status: string;
    workingDirectory: string;
    createdAt: string;
    updatedAt: string;
    wizardStep?: string | null;
  };
  wizardStep: string;
  highestStep: number;
  projectData: WizardProjectData;
  planComplete: boolean;
  stories: UserStory[];
  config: ProjectConfig | null;
}

export interface RawConfigInput {
  executeAgent: string;
  executeModel: string | null;
  executeEffort: string | null;
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

export interface SubmitConfigResult {
  config: ProjectConfig;
  errors: Record<string, string>;
}

export interface CompleteDescribeInput {
  projectId?: string | null;
  name: string;
  description: string;
  workingDirectory: string;
  connectionId?: string | null;
  planAgent: string;
  planModel?: string | null;
  planEffort?: string | null;
}

export interface WizardRouteResult {
  projectId: string;
  nextRoute: string;
  workingDirectory?: string | null;
}

export interface CompleteConfigureResult {
  config: ProjectConfig;
  errors: Record<string, string>;
  nextRoute: string;
}

export interface LaunchProjectResult {
  sessionId: string;
  monitorRoute: string;
}

export interface WizardStepMeta {
  number: number;
  label: string;
  slug: string;
}

export interface MonitorTabMeta {
  id: string;
  label: string;
  disableForInactive: boolean;
}

export interface WizardDefaultsResponse {
  defaultAgent: string;
  placeholderConfig: ProjectConfig;
  wizardSteps: WizardStepMeta[];
  monitorTabs: MonitorTabMeta[];
}
