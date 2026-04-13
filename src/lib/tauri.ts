export type { ResolvedAgentSelection } from "./ipc/agents";
export {
  checkSystemReadiness,
  detectAgents,
  getAgentCapabilities,
  getKnownAgents,
  refreshAgents,
  resolveAgentSelection,
} from "./ipc/agents";
export {
  askHistory,
  askQuestion,
  copyAskMessage,
  onAskComplete,
  onAskError,
  onAskStream,
  retryAsk,
  stopAsk,
  truncateAskFrom,
} from "./ipc/ask";
export {
  getAtomizerActivityLog,
  getAtomizerPipelineState,
  loadOutputLog,
  onAtomizationActivity,
  onAtomizationProgress,
  runAtomizer,
} from "./ipc/atomizer";
export type { DisplayVocabulary } from "./ipc/display-vocabulary";
export { getDisplayVocabulary } from "./ipc/display-vocabulary";
export type {
  AgentOutputPayload,
  HeartbeatPayload,
  LoopEvent,
  ProjectStateChangedPayload,
  PromptPayload,
  StorySkippedPayload,
  VerificationPayload,
} from "./ipc/loop";
export {
  ephemeralQuery,
  getActivityFeed,
  getIterationHistory,
  onAgentOutput,
  onAgentSwitched,
  onHeartbeat,
  onIterationCompleted,
  onIterationStarted,
  onProjectStateChanged,
  onPromptBuilt,
  onRateLimitDetected,
  onSessionEnded,
  onSessionStarted,
  onStoriesUpdated,
  onStorySkipped,
  onVerificationFailed,
  onVerificationPassed,
  onVerificationStarted,
  startLoop,
  stopLoop,
} from "./ipc/loop";
export type {
  AppNotification as BackendNotification,
  NotificationAddedPayload,
  NotificationListResponse,
  ProjectNotificationSummary,
  RingColor,
} from "./ipc/notifications";
export {
  addNotification,
  clearNotificationsBackend,
  getNotifications,
  markAllNotificationsRead,
  markNotificationRead,
  onNotificationAdded,
} from "./ipc/notifications";
export type {
  PlanStepState,
  PlanUserActionKind,
  PlanUserActionResult,
  ResolvePlanActionResult,
} from "./ipc/plan";
export {
  loadExistingPlan,
  loadExistingPrd,
  onPlanActivityBatch,
  onPlanComplete,
  onPlanError,
  onPlanHeartbeat,
  planUserAction,
  queryPlanStatus,
  resolvePlanAction,
  resolvePlanState,
  saveConfig,
  savePlan,
  savePrd,
  startPlan,
  stopPlan,
  writeToPlan,
} from "./ipc/plan";
export type { GroupedProjects } from "./ipc/project";
export {
  archiveProject,
  createProject,
  discardDraft,
  finalizeDraft,
  getProjectConfig,
  getProjectSnapshot,
  getProjectStories,
  listProjects,
  listProjectsEnriched,
  listProjectsGrouped,
  loadDraft,
  pauseProject,
  resumeProject,
  resumeWizard,
  saveDraft,
  saveWizardState,
} from "./ipc/project";
export type {
  AgentCapabilities,
  AgentEffortOption,
  AgentInfo,
  AgentModelOption,
  SystemReadiness,
  ArtifactPaths,
  AskCompletePayload,
  AskErrorPayload,
  AskMessage,
  AskQuestionResult,
  AskStreamPayload,
  AtomizeActivityPayload,
  AtomizeArgs,
  AtomizeProgress,
  EphemeralAnswer,
  IterationRow,
  IterationStory,
  PipelineSnapshot,
  PlanActivityBatchPayload,
  PlanActivityPayload,
  PlanSessionInfo,
  PlanSessionStatus,
  PlanTerminalPayload,
  Prd,
  ProgressInfo,
  ProjectConfig,
  ProjectRecord,
  ProjectSnapshot,
  ProjectsByStatus,
  SessionInfo,
  SnapshotStatus,
  StageSnapshot,
  StageStatus,
  StartLoopArgs,
  WizardResumeState,
} from "./ipc/types";

export type {
  AdvanceWizardResult,
  CompleteConfigureResult,
  CompleteDescribeInput,
  ConfigDefaultsResponse,
  ConfigLimits,
  LaunchProjectResult,
  LaunchReadiness,
  MonitorTabMeta,
  RawConfigInput,
  StoriesResponse,
  SubmitConfigResult,
  ValidationErrors,
  WizardDefaultsResponse,
  WizardHydrationResult,
  WizardRouteResult,
  WizardStepMeta,
} from "./ipc/wizard-logic";

export {
  addStory,
  advanceWizardStep,
  completeAtomizeStep,
  completeConfigureStep,
  completeDescribeStep,
  exitWizard,
  getDefaultConfig,
  getStories,
  getWizardDefaults,
  hydrateWizard,
  launchProject,
  markWizardStale,
  removeStoryBackend,
  reorderStoriesBackend,
  replan,
  saveWizardDraft,
  submitProjectConfig,
  updateStoryBackend,
  validateDescribeInput,
  validateLaunchReadiness,
  validateProjectConfig,
} from "./ipc/wizard-logic";

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
