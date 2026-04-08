export type PlanEventKind =
  | "search"
  | "docsLookup"
  | "mcpCall"
  | "planContent"
  | "thinking"
  | "error";

export interface PlanEvent {
  kind: PlanEventKind;
  content: string;
  timestamp: string;
}

export interface UserStory {
  id: string;
  title: string;
  description: string;
  acceptanceCriteria: string[];
  scope: {
    filesToModify: string[];
    filesToCreate: string[];
    filesToAvoid: string[];
  };
  verification: {
    commands: string[];
    assertions: string[];
  };
  commitMessage?: string;
  priority: "critical" | "high" | "medium" | "low";
  estimatedComplexity: "small" | "medium" | "large";
  estimatedMinutes: number;
  dependsOn: string[];
  passes: boolean;
  blocked: boolean;
  attempts: number;
  notes?: string | null;
}

export interface WizardProjectData {
  name: string;
  description: string;
  workingDirectory: string;
  planAgent: string;
  planModel: string | null;
  planEffort: string | null;
}

export interface WizardConfig {
  executeAgent: string;
  executeModel: string | null;
  executeEffort: string | null;
  fallbackChain: string[];
  gutterThreshold: number;
  maxIterations: number;
  cooldownSeconds: number;
  testCommand: string;
  maxVerificationRetries: number;
  scmProvider: "auto" | "github" | "gitlab" | "none";
  reviewPollingInterval: number;
  reviewTimeout: number;
}
