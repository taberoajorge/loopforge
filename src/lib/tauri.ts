export type {
  AgentInfo, AgentModelOption, AgentEffortOption, AgentCapabilities,
  ProjectRecord, ProjectsByStatus, ProjectConfig,
  SnapshotStatus, ArtifactPaths, SessionInfo, ProgressInfo,
  ProjectSnapshot, IterationStory, WizardResumeState,
  Prd, StartLoopArgs, AtomizeArgs, AtomizeProgress,
  IterationRow, EphemeralAnswer,
  PlanActivityPayload, PlanTerminalPayload, PlanActivityBatchPayload,
  PlanSessionStatus, PlanSessionInfo,
  AskMessage, AskStreamPayload, AskCompletePayload, AskErrorPayload,
} from "./ipc/types";

export type {
  HeartbeatPayload, VerificationPayload, PromptPayload,
  StorySkippedPayload, AgentOutputPayload, LoopEvent,
} from "./ipc/loop";

export { detectAgents, refreshAgents, getAgentCapabilities } from "./ipc/agents";

export {
  listProjects, listProjectsEnriched, createProject, getProjectSnapshot,
  pauseProject, resumeProject, getProjectStories, getProjectConfig,
  saveWizardState, saveDraft, loadDraft, resumeWizard,
  finalizeDraft, discardDraft, archiveProject,
} from "./ipc/project";

export {
  startPlan, writeToPlan, stopPlan, queryPlanStatus,
  loadExistingPlan, savePlan, loadExistingPrd, savePrd, saveConfig,
  onPlanActivityBatch, onPlanComplete, onPlanError, onPlanHeartbeat,
} from "./ipc/plan";

export { runAtomizer, loadOutputLog, onAtomizationProgress } from "./ipc/atomizer";

export {
  askQuestion, askHistory, stopAsk, copyAskMessage,
  truncateAskFrom, retryAsk, onAskStream, onAskComplete, onAskError,
} from "./ipc/ask";

export {
  startLoop, stopLoop, getIterationHistory, ephemeralQuery,
  onAgentOutput, onIterationStarted, onIterationCompleted,
  onSessionStarted, onSessionEnded, onRateLimitDetected,
  onAgentSwitched, onHeartbeat, onVerificationStarted,
  onVerificationFailed, onVerificationPassed,
  onPromptBuilt, onStorySkipped,
} from "./ipc/loop";

export const tauriBridgeMode = "legacy-compatibility";

export function isTauriBridgeCompatibilityEnabled(): boolean {
  if (typeof window === "undefined") {
    return false;
  }
  const isLegacyMode = import.meta.env.MODE === "legacy-router";
  const hasLegacyQuery = new URLSearchParams(window.location.search).has("legacy-router");
  return isLegacyMode || hasLegacyQuery;
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
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<Connection[]>("list_connections");
}

export async function buildConnectionWorkspace(connectionId: string): Promise<string> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<string>("build_connection_workspace", { connectionId });
}
