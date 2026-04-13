import type { DragEvent } from "react";
import type { AgentCapabilities } from "../../lib/tauri";
import type { ConfigureErrors } from "./configureValidation";

export type ScmProvider = "auto" | "github" | "gitlab" | "none";

export interface ConfigureValues {
  executeAgent: string;
  executeModel: string | null;
  executeEffort: string | null;
  fallbackChain: string[];
  gutterThreshold: number;
  maxIterations: number;
  cooldownSeconds: number;
  testCommand: string;
  maxVerificationRetries: number;
  scmProvider: ScmProvider;
  reviewPollingInterval: number;
  reviewTimeout: number;
  newAgent: string;
}

export interface ConfigureFormOptions {
  capabilities: AgentCapabilities | null;
  errors: ConfigureErrors;
  selectableAgentNames: string[];
  agentsNotInChain: string[];
}

export interface ConfigureFormHandlers {
  onValueChange: (
    field: keyof ConfigureValues,
    value: ConfigureValues[keyof ConfigureValues],
  ) => void;
  onAddAgentToChain: () => void;
  onRemoveFromChain: (agentName: string) => void;
  onDragStart: (index: number) => void;
  onDragOver: (event: DragEvent<HTMLDivElement>) => void;
  onDrop: (index: number) => void;
  onNext: () => void;
}
