import type {
  ArtifactPaths,
  ProgressInfo,
  ProjectConfig,
  ProjectRecord,
  ProjectSnapshot,
  ProjectsByStatus,
  SessionInfo,
  WizardResumeState,
} from "../../lib/tauri";
import type { Project } from "../../types/project";
import { type DeepPartial, mergeFixture } from "./shared";

export function createProject(overrides?: DeepPartial<Project>): Project {
  return mergeFixture<Project>(
    {
      id: "project-001",
      name: "LoopForge",
      description: "Desktop AI loop orchestrator",
      status: "draft",
      workingDirectory: "/work/loopforge",
      createdAt: "2026-04-09T10:00:00.000Z",
      updatedAt: "2026-04-09T10:00:00.000Z",
      storiesCompleted: 0,
      totalStories: 3,
      currentAgent: null,
      sessionStartedAt: null,
      sessionEndedAt: null,
      durationLabel: null,
      uptimeLabel: null,
      wizardStep: "describe",
    },
    overrides,
  );
}

export function createProjectRecord(overrides?: DeepPartial<ProjectRecord>): ProjectRecord {
  return mergeFixture<ProjectRecord>(
    {
      id: "project-001",
      name: "LoopForge",
      description: "Desktop AI loop orchestrator",
      status: "draft",
      workingDirectory: "/work/loopforge",
      createdAt: "2026-04-09T10:00:00.000Z",
      updatedAt: "2026-04-09T10:00:00.000Z",
    },
    overrides,
  );
}

export function createProjectsByStatus(
  overrides?: DeepPartial<ProjectsByStatus>,
): ProjectsByStatus {
  return mergeFixture<ProjectsByStatus>(
    {
      active: [],
      paused: [],
      completed: [],
      draft: [createProjectRecord()],
      archived: [],
      blocked: [],
      failed: [],
    },
    overrides,
  );
}

export function createProjectConfig(overrides?: DeepPartial<ProjectConfig>): ProjectConfig {
  return mergeFixture<ProjectConfig>(
    {
      schemaVersion: 1,
      executeAgent: "codex",
      executeModel: "gpt-5.4",
      executeEffort: "medium",
      fallbackChain: ["claude"],
      gutterThreshold: 3,
      maxIterations: 50,
      cooldownSeconds: 5,
      testCommand: "bun run test",
      maxVerificationRetries: 3,
      scmProvider: "github",
      reviewPollingInterval: 60,
      reviewTimeout: 600,
    },
    overrides,
  );
}

export function createProjectSnapshot(overrides?: DeepPartial<ProjectSnapshot>): ProjectSnapshot {
  const artifactPaths: ArtifactPaths = {
    root: "/config/loopforge/project-001",
    draft: "/config/loopforge/project-001/draft.json",
    plan: "/config/loopforge/project-001/plan.md",
    prd: "/config/loopforge/project-001/prd.json",
    config: "/config/loopforge/project-001/config.json",
    prompt: "/config/loopforge/project-001/prompt.md",
    guardrails: "/config/loopforge/project-001/guardrails.md",
  };
  const activeSession: SessionInfo = {
    id: "session-001",
    startedAt: "2026-04-09T10:15:00.000Z",
    endedAt: null,
  };
  const progress: ProgressInfo = {
    storiesTotal: 3,
    storiesDone: 1,
    currentStory: "S-005",
  };
  return mergeFixture<ProjectSnapshot>(
    {
      project: {
        ...createProjectRecord(),
        wizardStep: "monitor",
      },
      status: "running",
      activeSession,
      progress,
      config: createProjectConfig(),
      artifactPaths,
      progressPercent: 33,
      uptimeLabel: "15m",
    },
    overrides,
  );
}

export function createWizardResumeState(
  overrides?: DeepPartial<WizardResumeState>,
): WizardResumeState {
  return mergeFixture<WizardResumeState>(
    {
      project: createProjectRecord(),
      wizardStep: "plan",
      wizardStateJson: '{"currentStep":2}',
      hasPlan: true,
      hasPrd: false,
    },
    overrides,
  );
}
