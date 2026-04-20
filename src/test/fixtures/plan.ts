import type {
  PlanActivityBatchPayload,
  PlanActivityPayload,
  PlanSessionInfo,
  PlanTerminalPayload,
} from "../../lib/tauri";
import { type DeepPartial, mergeFixture } from "./shared";

export function createPlanActivityPayload(
  overrides?: DeepPartial<PlanActivityPayload>,
): PlanActivityPayload {
  return mergeFixture<PlanActivityPayload>(
    {
      projectId: "project-001",
      kind: "thinking",
      content: "Planning work",
      timestamp: "2026-04-09T10:20:00.000Z",
    },
    overrides,
  );
}

export function createPlanActivityBatchPayload(
  overrides?: DeepPartial<PlanActivityBatchPayload>,
): PlanActivityBatchPayload {
  return mergeFixture<PlanActivityBatchPayload>(
    {
      projectId: "project-001",
      events: [createPlanActivityPayload()],
      planContent: "Define the scope.",
      planContentDelta: "Define the scope.",
    },
    overrides,
  );
}

export function createPlanTerminalPayload(
  overrides?: DeepPartial<PlanTerminalPayload>,
): PlanTerminalPayload {
  return mergeFixture<PlanTerminalPayload>(
    {
      projectId: "project-001",
      detail: "Plan completed",
    },
    overrides,
  );
}

export function createPlanSessionInfo(overrides?: DeepPartial<PlanSessionInfo>): PlanSessionInfo {
  return mergeFixture<PlanSessionInfo>(
    {
      status: "running",
      agentName: "codex",
      elapsedSecs: 12,
      lastActivitySecsAgo: 1,
    },
    overrides,
  );
}
