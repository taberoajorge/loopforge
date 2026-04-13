import type {
  AgentCapabilities,
  AgentInfo,
  AgentOutputPayload,
  AskCompletePayload,
  AskErrorPayload,
  AskMessage,
  AskQuestionResult,
  AskStreamPayload,
  AtomizeArgs,
  AtomizeProgress,
  CompleteConfigureResult,
  ConfigDefaultsResponse,
  Connection,
  DisplayVocabulary,
  EphemeralAnswer,
  GroupedProjects,
  HeartbeatPayload,
  IterationRow,
  IterationStory,
  NotificationAddedPayload,
  PipelineSnapshot,
  PlanActivityBatchPayload,
  PlanSessionInfo,
  PlanTerminalPayload,
  Prd,
  ProjectConfig,
  ProjectRecord,
  ProjectSnapshot,
  ProjectStateChangedPayload,
  ProjectsByStatus,
  PromptPayload,
  RawConfigInput,
  ResolvedAgentSelection,
  StartLoopArgs,
  StorySkippedPayload,
  VerificationPayload,
  WizardResumeState,
} from "../../lib/tauri";
import type { Project } from "../../types/project";

export interface ProjectScopedEventPayload {
  projectId: string;
  [key: string]: unknown;
}

export interface TauriCommandMap {
  detect_agents: { args: undefined; result: AgentInfo[] };
  refresh_agents: { args: undefined; result: AgentInfo[] };
  get_known_agents: { args: undefined; result: string[] };
  get_default_config: { args: undefined; result: ConfigDefaultsResponse };
  get_agent_capabilities: { args: { agent: string }; result: AgentCapabilities };
  resolve_agent_selection: {
    args: { agent: string; currentModel?: string | null; currentEffort?: string | null };
    result: ResolvedAgentSelection;
  };
  get_display_vocabulary: { args: undefined; result: DisplayVocabulary };
  list_projects: { args: undefined; result: ProjectsByStatus };
  list_projects_enriched: { args: undefined; result: Project[] };
  list_projects_grouped: { args: undefined; result: GroupedProjects };
  create_project: {
    args: { name: string; description: string; workingDirectory: string; wizardStep?: string };
    result: ProjectRecord;
  };
  get_project_snapshot: { args: { projectId: string }; result: ProjectSnapshot };
  pause_project: { args: { projectId: string }; result: undefined };
  resume_project: { args: { projectId: string }; result: undefined };
  get_project_stories: { args: { projectId: string }; result: IterationStory[] };
  get_project_config: { args: { projectId: string }; result: ProjectConfig | null };
  save_wizard_state: {
    args: { projectId: string; wizardStep: string; wizardStateJson: string };
    result: undefined;
  };
  save_draft: { args: { projectId: string; draftJson: string }; result: undefined };
  load_draft: { args: { projectId: string }; result: string | null };
  resume_wizard: { args: { projectId: string }; result: WizardResumeState };
  finalize_draft: { args: { projectId: string }; result: undefined };
  discard_draft: { args: { projectId: string }; result: undefined };
  archive_project: { args: { projectId: string }; result: undefined };
  start_plan: { args: { args: StartPlanArgs }; result: undefined };
  write_to_plan: { args: { projectId: string; input: string }; result: undefined };
  stop_plan: { args: { projectId: string }; result: undefined };
  query_plan_status: { args: { projectId: string }; result: PlanSessionInfo | null };
  load_existing_plan: { args: { projectId: string }; result: string | null };
  save_plan: { args: { projectId: string; content: string }; result: undefined };
  load_existing_prd: { args: { projectId: string }; result: Prd | null };
  save_prd: { args: { projectId: string; prdJson: string }; result: undefined };
  save_config: { args: { projectId: string; configJson: string }; result: undefined };
  complete_configure_step: {
    args: { projectId: string; raw: RawConfigInput };
    result: CompleteConfigureResult;
  };
  run_atomizer: { args: { args: AtomizeArgs }; result: Prd };
  get_atomizer_pipeline_state: { args: { projectId: string }; result: PipelineSnapshot | null };
  load_output_log: { args: { projectId: string }; result: string };
  ask_question: { args: { args: AskQuestionArgs }; result: AskQuestionResult };
  ask_history: { args: { projectId: string }; result: AskMessage[] };
  stop_ask: { args: { projectId: string }; result: undefined };
  copy_ask_message: { args: { messageId: string }; result: string };
  truncate_ask_from: { args: { projectId: string; messageId: string }; result: AskMessage[] };
  retry_ask: { args: RetryAskArgs; result: string };
  start_loop: { args: { args: StartLoopArgs }; result: string };
  stop_loop: { args: { projectId: string }; result: undefined };
  get_iteration_history: { args: { projectId: string }; result: IterationRow[] };
  get_activity_feed: { args: { projectId: string }; result: IterationRow[] };
  ephemeral_query: { args: { projectId: string; question: string }; result: EphemeralAnswer };
  list_connections: { args: undefined; result: Connection[] };
  build_connection_workspace: { args: { connectionId: string }; result: string };
}

export interface TauriEventMap {
  "plan:activity-batch": PlanActivityBatchPayload;
  "plan:complete": PlanTerminalPayload;
  "plan:error": PlanTerminalPayload;
  "plan:heartbeat": PlanTerminalPayload;
  "atomization-progress": AtomizeProgress;
  "ask:stream": AskStreamPayload;
  "ask:complete": AskCompletePayload;
  "ask:error": AskErrorPayload;
  "agent-output-stream": AgentOutputPayload;
  "loop:iteration-started": ProjectScopedEventPayload;
  "loop:iteration-completed": ProjectScopedEventPayload;
  "loop:session-started": ProjectScopedEventPayload;
  "loop:session-ended": ProjectScopedEventPayload;
  "loop:rate-limit-detected": ProjectScopedEventPayload;
  "loop:agent-switched": ProjectScopedEventPayload;
  "loop:heartbeat": HeartbeatPayload;
  "loop:verification-started": VerificationPayload;
  "loop:verification-failed": VerificationPayload;
  "loop:verification-passed": VerificationPayload;
  "loop:prompt-built": PromptPayload;
  "loop:story-skipped": StorySkippedPayload;
  "project:state-changed": ProjectStateChangedPayload;
  "notification:added": NotificationAddedPayload;
}

export interface StartPlanArgs {
  projectId: string;
  projectDir: string;
  agent: string;
  model?: string | null;
  effort?: string | null;
  initialPrompt: string;
}

export interface AskQuestionArgs {
  projectId: string;
  question: string;
  agent: string;
  model?: string | null;
}

export interface RetryAskArgs {
  projectId: string;
  messageId: string;
  agent: string;
  model?: string | null;
}

export type TauriCommandName = keyof TauriCommandMap;
export type TauriEventName = keyof TauriEventMap;
export type TauriCommandArgs<TName extends TauriCommandName> = TauriCommandMap[TName]["args"];
export type TauriCommandResult<TName extends TauriCommandName> = TauriCommandMap[TName]["result"];
export type TauriEventPayload<TName extends TauriEventName> = TauriEventMap[TName];
