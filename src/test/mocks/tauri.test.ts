import { expect, test } from "vitest";
import {
  detectAgents,
  onPlanActivityBatch,
} from "../../lib/tauri";
import {
  createAgentInfo,
  createPlanActivityBatchPayload,
  createProjectSnapshot,
} from "../fixtures";
import {
  emitTauriEvent,
  getEventListenerCount,
  invokeMock,
  mockTauriCommand,
} from "../mocks";

test("mocks IPC commands through the real frontend wrappers", async () => {
  const mockedAgents = [createAgentInfo()];
  mockTauriCommand("detect_agents", mockedAgents);
  mockTauriCommand("get_project_snapshot", createProjectSnapshot());

  await expect(detectAgents()).resolves.toEqual(mockedAgents);
  expect(invokeMock).toHaveBeenCalledWith("detect_agents");
});

test("emits typed frontend events to subscribed listeners", async () => {
  const payload = createPlanActivityBatchPayload();
  const received: typeof payload[] = [];
  const unlisten = await onPlanActivityBatch((value) => {
    received.push(value);
  });

  expect(getEventListenerCount("plan:activity-batch")).toBe(1);
  await emitTauriEvent("plan:activity-batch", payload);
  expect(received).toEqual([payload]);

  unlisten();
  expect(getEventListenerCount("plan:activity-batch")).toBe(0);
});
