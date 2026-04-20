import type { AgentCapabilities, AgentInfo, Prd } from "../../lib/tauri";
import type { PlanEvent, UserStory, WizardConfig, WizardProjectData } from "../../types/wizard";
import { type DeepPartial, mergeFixture } from "./shared";

export function createAgentInfo(overrides?: DeepPartial<AgentInfo>): AgentInfo {
  return mergeFixture<AgentInfo>(
    {
      name: "codex",
      binary: "codex",
      version: "0.118.0",
      available: true,
    },
    overrides,
  );
}

export function createAgentCapabilities(
  overrides?: DeepPartial<AgentCapabilities>,
): AgentCapabilities {
  return mergeFixture<AgentCapabilities>(
    {
      agent: "codex",
      source: "local",
      supportsModel: true,
      supportsEffort: true,
      models: [{ id: "gpt-5.4", label: "GPT-5.4" }],
      efforts: [{ id: "medium", label: "Medium" }],
      defaultModel: "gpt-5.4",
      defaultEffort: "medium",
    },
    overrides,
  );
}

export function createWizardProjectData(
  overrides?: DeepPartial<WizardProjectData>,
): WizardProjectData {
  return mergeFixture<WizardProjectData>(
    {
      name: "LoopForge Refactor",
      description: "Refactor frontend test support",
      workingDirectory: "/work/loopforge",
      planAgent: "codex",
      planModel: "gpt-5.4",
      planEffort: "medium",
    },
    overrides,
  );
}

export function createWizardConfig(overrides?: DeepPartial<WizardConfig>): WizardConfig {
  return mergeFixture<WizardConfig>(
    {
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

export function createPlanEvent(overrides?: DeepPartial<PlanEvent>): PlanEvent {
  return mergeFixture<PlanEvent>(
    {
      kind: "thinking",
      content: "Inspecting the codebase",
      timestamp: "2026-04-09T10:10:00.000Z",
    },
    overrides,
  );
}

export function createUserStory(overrides?: DeepPartial<UserStory>): UserStory {
  return mergeFixture<UserStory>(
    {
      id: "S-005",
      title: "Create typed IPC and event test doubles",
      description: "Provide typed frontend test helpers",
      acceptanceCriteria: ["Helpers are reusable", "Fixtures support overrides"],
      scope: {
        filesToModify: ["src/test/setup.ts"],
        filesToCreate: ["src/test/mocks/index.ts"],
        filesToAvoid: [],
      },
      verification: {
        commands: ["bun run typecheck", "bun run test"],
        assertions: [],
      },
      commitMessage: "feat(frontend): add test doubles",
      priority: "critical",
      estimatedComplexity: "medium",
      estimatedMinutes: 50,
      dependsOn: ["S-004"],
      passes: false,
      blocked: false,
      attempts: 0,
      notes: null,
    },
    overrides,
  );
}

export function createPrd(overrides?: DeepPartial<Prd>): Prd {
  return mergeFixture<Prd>(
    {
      projectName: "LoopForge",
      generatedAt: "2026-04-09T10:00:00.000Z",
      totalEstimatedMinutes: 50,
      stories: [createUserStory()],
    },
    overrides,
  );
}
