import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { PipelineSnapshot } from "../lib/tauri";
import { useWizardStore } from "../stores/wizardStore";
import {
  createAtomizedPrd,
  createAtomizeProgress,
  createWizardProjectData,
} from "../test/fixtures";
import { emitTauriEvent, mockTauriCommands } from "../test/mocks";
import { useAtomizerPipeline } from "./useAtomizerPipeline";

function createDeferred<TValue>() {
  let resolve!: (value: TValue) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<TValue>((nextResolve, nextReject) => {
    resolve = nextResolve;
    reject = nextReject;
  });
  return { promise, resolve, reject };
}

function resetAtomizerStore() {
  useWizardStore.getState().reset();
  useWizardStore.setState({
    projectId: "project-001",
    projectData: createWizardProjectData(),
  });
}

describe("useAtomizerPipeline", () => {
  beforeEach(() => {
    resetAtomizerStore();
  });

  it("tracks stage transitions and applies the completed prd payload", async () => {
    const atomizer = createDeferred<ReturnType<typeof createAtomizedPrd>>();
    let snapshotState: PipelineSnapshot | null = null;
    mockTauriCommands({
      get_atomizer_pipeline_state: vi.fn(async () => snapshotState),
      run_atomizer: vi.fn(() => atomizer.promise),
    });

    const { result } = renderHook(() => useAtomizerPipeline("project-001"));

    await waitFor(() => expect(result.current.atomizeStarted).toBe(true));
    expect(result.current.stages.map((stage) => stage.status)).toEqual([
      "pending",
      "pending",
      "pending",
      "pending",
    ]);

    await act(async () => {
      snapshotState = {
        stages: [
          { number: 1, label: "Summarize", status: "done" },
          { number: 2, label: "Chunk", status: "done" },
          { number: 3, label: "Atomize", status: "running" },
          { number: 4, label: "Merge", status: "pending" },
        ],
        error: null,
        startedAt: "2026-04-09T10:15:00.000Z",
        elapsedMs: 3000,
        done: false,
        percent: 62,
        isDone: false,
        isRunning: true,
      };
      await emitTauriEvent(
        "atomization-progress",
        createAtomizeProgress({
          stage: 3,
          stageName: "Atomize",
          message: "Atomizing stories",
        }),
      );
    });

    expect(result.current.stageMessage).toBe("Atomizing stories");
    expect(result.current.stages.map((stage) => stage.status)).toEqual([
      "done",
      "done",
      "running",
      "pending",
    ]);

    await act(async () => {
      snapshotState = {
        stages: [
          { number: 1, label: "Summarize", status: "done" },
          { number: 2, label: "Chunk", status: "done" },
          { number: 3, label: "Atomize", status: "done" },
          { number: 4, label: "Merge", status: "done" },
        ],
        error: null,
        startedAt: "2026-04-09T10:15:00.000Z",
        elapsedMs: 5000,
        done: true,
        percent: 100,
        isDone: true,
        isRunning: false,
      };
      atomizer.resolve(createAtomizedPrd());
      await atomizer.promise;
    });

    await waitFor(() => expect(result.current.isDone).toBe(true));
    expect(result.current.stageMessage).toBe("Done. 2 stories generated.");
    expect(useWizardStore.getState().stories).toEqual(createAtomizedPrd().stories);
  });

  it("marks the active stage as errored when atomization fails", async () => {
    let snapshotState: PipelineSnapshot | null = null;
    mockTauriCommands({
      get_atomizer_pipeline_state: vi.fn(async () => snapshotState),
      run_atomizer: vi.fn(async () => {
        snapshotState = {
          stages: [
            { number: 1, label: "Summarize", status: "error" },
            { number: 2, label: "Chunk", status: "pending" },
            { number: 3, label: "Atomize", status: "pending" },
            { number: 4, label: "Merge", status: "pending" },
          ],
          error: "Atomizer failed",
          startedAt: "2026-04-09T10:15:00.000Z",
          elapsedMs: 1500,
          done: false,
          percent: 12,
          isDone: false,
          isRunning: false,
        };
        return Promise.reject(new Error("Atomizer failed"));
      }),
    });

    const { result } = renderHook(() => useAtomizerPipeline("project-001"));

    await waitFor(() => expect(result.current.atomizeError).toBe("Atomizer failed"));
    expect(result.current.stages.map((stage) => stage.status)).toEqual([
      "error",
      "pending",
      "pending",
      "pending",
    ]);
    expect(result.current.isRunning).toBe(false);
  });

  it("uses backend snapshot and skips atomizer when already done", async () => {
    const runAtomizerCommand = vi.fn(async () => createAtomizedPrd());
    mockTauriCommands({
      get_atomizer_pipeline_state: {
        stages: [
          { number: 1, label: "Summarize", status: "done" },
          { number: 2, label: "Chunk", status: "done" },
          { number: 3, label: "Atomize", status: "done" },
          { number: 4, label: "Merge", status: "done" },
        ],
        error: null,
        startedAt: "2026-04-09T10:15:00.000Z",
        elapsedMs: 5000,
        done: true,
        percent: 100,
        isDone: true,
        isRunning: false,
      },
      run_atomizer: runAtomizerCommand,
    });

    const { result } = renderHook(() => useAtomizerPipeline("project-001"));

    await waitFor(() => expect(result.current.isDone).toBe(true));
    expect(result.current.stages.every((stage) => stage.status === "done")).toBe(true);
    expect(runAtomizerCommand).not.toHaveBeenCalled();
  });
});
