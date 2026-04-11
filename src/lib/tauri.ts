import { invoke } from "@tauri-apps/api/core";
import * as agentBridge from "./ipc/agents";
import * as askBridge from "./ipc/ask";
import * as atomizerBridge from "./ipc/atomizer";
import * as loopBridge from "./ipc/loop";
import * as planBridge from "./ipc/plan";
import * as projectBridge from "./ipc/project";

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

const legacyRouterQueryKey = "legacy-router";
export const tauriBridgeMode = "legacy-compatibility";

export function isTauriBridgeCompatibilityEnabled(): boolean {
  if (typeof window === "undefined") {
    return false;
  }
  const isLegacyMode = import.meta.env.MODE === "legacy-router";
  const hasLegacyQuery = new URLSearchParams(window.location.search).has(legacyRouterQueryKey);
  return isLegacyMode || hasLegacyQuery;
}

function assertLegacyCompatibility(apiName: string): void {
  if (isTauriBridgeCompatibilityEnabled()) {
    return;
  }
  throw new Error(`${apiName} requires legacy router compatibility mode.`);
}

function withLegacyCompatibility<Arguments extends unknown[], Result>(
  apiName: string,
  operation: (...argumentsList: Arguments) => Result,
): (...argumentsList: Arguments) => Result {
  return (...argumentsList: Arguments): Result => {
    assertLegacyCompatibility(apiName);
    return operation(...argumentsList);
  };
}

export const detectAgents = withLegacyCompatibility("detectAgents", agentBridge.detectAgents);
export const refreshAgents = withLegacyCompatibility("refreshAgents", agentBridge.refreshAgents);
export const getAgentCapabilities = withLegacyCompatibility("getAgentCapabilities", agentBridge.getAgentCapabilities);

export const listProjects = withLegacyCompatibility("listProjects", projectBridge.listProjects);
export const listProjectsEnriched = withLegacyCompatibility("listProjectsEnriched", projectBridge.listProjectsEnriched);
export const createProject = withLegacyCompatibility("createProject", projectBridge.createProject);
export const getProjectSnapshot = withLegacyCompatibility("getProjectSnapshot", projectBridge.getProjectSnapshot);
export const pauseProject = withLegacyCompatibility("pauseProject", projectBridge.pauseProject);
export const resumeProject = withLegacyCompatibility("resumeProject", projectBridge.resumeProject);
export const getProjectStories = withLegacyCompatibility("getProjectStories", projectBridge.getProjectStories);
export const getProjectConfig = withLegacyCompatibility("getProjectConfig", projectBridge.getProjectConfig);
export const saveWizardState = withLegacyCompatibility("saveWizardState", projectBridge.saveWizardState);
export const saveDraft = withLegacyCompatibility("saveDraft", projectBridge.saveDraft);
export const loadDraft = withLegacyCompatibility("loadDraft", projectBridge.loadDraft);
export const resumeWizard = withLegacyCompatibility("resumeWizard", projectBridge.resumeWizard);
export const finalizeDraft = withLegacyCompatibility("finalizeDraft", projectBridge.finalizeDraft);
export const discardDraft = withLegacyCompatibility("discardDraft", projectBridge.discardDraft);
export const archiveProject = withLegacyCompatibility("archiveProject", projectBridge.archiveProject);

export const startPlan = withLegacyCompatibility("startPlan", planBridge.startPlan);
export const writeToPlan = withLegacyCompatibility("writeToPlan", planBridge.writeToPlan);
export const stopPlan = withLegacyCompatibility("stopPlan", planBridge.stopPlan);
export const queryPlanStatus = withLegacyCompatibility("queryPlanStatus", planBridge.queryPlanStatus);
export const loadExistingPlan = withLegacyCompatibility("loadExistingPlan", planBridge.loadExistingPlan);
export const savePlan = withLegacyCompatibility("savePlan", planBridge.savePlan);
export const loadExistingPrd = withLegacyCompatibility("loadExistingPrd", planBridge.loadExistingPrd);
export const savePrd = withLegacyCompatibility("savePrd", planBridge.savePrd);
export const saveConfig = withLegacyCompatibility("saveConfig", planBridge.saveConfig);
export const onPlanActivityBatch = withLegacyCompatibility("onPlanActivityBatch", planBridge.onPlanActivityBatch);
export const onPlanComplete = withLegacyCompatibility("onPlanComplete", planBridge.onPlanComplete);
export const onPlanError = withLegacyCompatibility("onPlanError", planBridge.onPlanError);
export const onPlanHeartbeat = withLegacyCompatibility("onPlanHeartbeat", planBridge.onPlanHeartbeat);

export const runAtomizer = withLegacyCompatibility("runAtomizer", atomizerBridge.runAtomizer);
export const loadOutputLog = withLegacyCompatibility("loadOutputLog", atomizerBridge.loadOutputLog);
export const onAtomizationProgress = withLegacyCompatibility("onAtomizationProgress", atomizerBridge.onAtomizationProgress);

export const askQuestion = withLegacyCompatibility("askQuestion", askBridge.askQuestion);
export const askHistory = withLegacyCompatibility("askHistory", askBridge.askHistory);
export const stopAsk = withLegacyCompatibility("stopAsk", askBridge.stopAsk);
export const copyAskMessage = withLegacyCompatibility("copyAskMessage", askBridge.copyAskMessage);
export const truncateAskFrom = withLegacyCompatibility("truncateAskFrom", askBridge.truncateAskFrom);
export const retryAsk = withLegacyCompatibility("retryAsk", askBridge.retryAsk);
export const onAskStream = withLegacyCompatibility("onAskStream", askBridge.onAskStream);
export const onAskComplete = withLegacyCompatibility("onAskComplete", askBridge.onAskComplete);
export const onAskError = withLegacyCompatibility("onAskError", askBridge.onAskError);

export const startLoop = withLegacyCompatibility("startLoop", loopBridge.startLoop);
export const stopLoop = withLegacyCompatibility("stopLoop", loopBridge.stopLoop);
export const getIterationHistory = withLegacyCompatibility("getIterationHistory", loopBridge.getIterationHistory);
export const ephemeralQuery = withLegacyCompatibility("ephemeralQuery", loopBridge.ephemeralQuery);
export const onAgentOutput = withLegacyCompatibility("onAgentOutput", loopBridge.onAgentOutput);
export const onIterationStarted = withLegacyCompatibility("onIterationStarted", loopBridge.onIterationStarted);
export const onIterationCompleted = withLegacyCompatibility("onIterationCompleted", loopBridge.onIterationCompleted);
export const onSessionStarted = withLegacyCompatibility("onSessionStarted", loopBridge.onSessionStarted);
export const onSessionEnded = withLegacyCompatibility("onSessionEnded", loopBridge.onSessionEnded);
export const onRateLimitDetected = withLegacyCompatibility("onRateLimitDetected", loopBridge.onRateLimitDetected);
export const onAgentSwitched = withLegacyCompatibility("onAgentSwitched", loopBridge.onAgentSwitched);
export const onHeartbeat = withLegacyCompatibility("onHeartbeat", loopBridge.onHeartbeat);
export const onVerificationStarted = withLegacyCompatibility("onVerificationStarted", loopBridge.onVerificationStarted);
export const onVerificationFailed = withLegacyCompatibility("onVerificationFailed", loopBridge.onVerificationFailed);
export const onVerificationPassed = withLegacyCompatibility("onVerificationPassed", loopBridge.onVerificationPassed);
export const onPromptBuilt = withLegacyCompatibility("onPromptBuilt", loopBridge.onPromptBuilt);
export const onStorySkipped = withLegacyCompatibility("onStorySkipped", loopBridge.onStorySkipped);

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

const listConnectionsBridge = (): Promise<Connection[]> => invoke<Connection[]>("list_connections");
const buildConnectionWorkspaceBridge = (connectionId: string): Promise<string> => (
  invoke<string>("build_connection_workspace", { connectionId })
);

export const listConnections = withLegacyCompatibility("listConnections", listConnectionsBridge);
export const buildConnectionWorkspace = withLegacyCompatibility("buildConnectionWorkspace", buildConnectionWorkspaceBridge);
