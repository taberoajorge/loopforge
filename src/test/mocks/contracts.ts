import type {
  AgentCapabilities,
  AgentInfo,
  AskCompletePayload,
  AskErrorPayload,
  AskMessage,
  AskStreamPayload,
  AtomizeArgs,
  AtomizeProgress,
  Connection,
  EphemeralAnswer,
  IterationRow,
  IterationStory,
  PlanActivityBatchPayload,
  PlanSessionInfo,
  PlanTerminalPayload,
  Prd,
  ProjectConfig,
  ProjectRecord,
  ProjectSnapshot,
  ProjectsByStatus,
  StartLoopArgs,
  WizardResumeState,
} from "../../lib/tauri";
import type {
  AgentOutputPayload,
  HeartbeatPayload,
  PromptPayload,
  StorySkippedPayload,
  VerificationPayload,
} from "../../lib/tauri";
import type { Project } from "../../types/project";

export interface ProjectScopedEventPayload {
  projectId: string;
  [key: string]: unknown;
}

export interface TauriCommandMap {
  detect_agents: { args: undefined; result: AgentInfo[] };
  refresh_agents: { args: undefined; result: AgentInfo[] };
  get_agent_capabilities: { args: { agent: string }; result: AgentCapabilities };
  list_projects: { args: undefined; result: ProjectsByStatus };
  list_projects_enriched: { args: undefined; result: Project[] };
  create_project: {
    args: { name: string; description: string; workingDirectory: string; wizardStep?: string };
    result: ProjectRecord;
  };
  get_project_snapshot: { args: { projectId: string }; result: ProjectSnapshot };
  pause_project: { args: { projectId: string }; result: void };
  resume_project: { args: { projectId: string }; result: void };
  get_project_stories: { args: { projectId: string }; result: IterationStory[] };
  get_project_config: { args: { projectId: string }; result: ProjectConfig | null };
  save_wizard_state: {
    args: { projectId: string; wizardStep: string; wizardStateJson: string };
    result: void;
  };
  save_draft: { args: { projectId: string; draftJson: string }; result: void };
  load_draft: { args: { projectId: string }; result: string | null };
  resume_wizard: { args: { projectId: string }; result: WizardResumeState };
  finalize_draft: { args: { projectId: string }; result: void };
  discard_draft: { args: { projectId: string }; result: void };
  archive_project: { args: { projectId: string }; result: void };
  start_plan: { args: { args: StartPlanArgs }; result: void };
  write_to_plan: { args: { projectId: string; input: string }; result: void };
  stop_plan: { args: { projectId: string }; result: void };
  query_plan_status: { args: { projectId: string }; result: PlanSessionInfo | null };
  load_existing_plan: { args: { projectId: string }; result: string | null };
  save_plan: { args: { projectId: string; content: string }; result: void };
  load_existing_prd: { args: { projectId: string }; result: Prd | null };
  save_prd: { args: { projectId: string; prdJson: string }; result: void };
  save_config: { args: { projectId: string; configJson: string }; result: void };
  run_atomizer: { args: { args: AtomizeArgs }; result: Prd };
  load_output_log: { args: { projectId: string }; result: string };
  ask_question: { args: { args: AskQuestionArgs }; result: string };
  ask_history: { args: { projectId: string }; result: AskMessage[] };
  stop_ask: { args: { projectId: string }; result: void };
  copy_ask_message: { args: { messageId: string }; result: string };
  truncate_ask_from: { args: { projectId: string; messageId: string }; result: AskMessage[] };
  retry_ask: { args: RetryAskArgs; result: string };
  start_loop: { args: { args: StartLoopArgs }; result: string };
  stop_loop: { args: { projectId: string }; result: void };
  get_iteration_history: { args: { projectId: string }; result: IterationRow[] };
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
