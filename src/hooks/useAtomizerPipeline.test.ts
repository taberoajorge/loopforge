import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  createAtomizeProgress,
  createAtomizedPrd,
  createWizardProjectData,
} from "../test/fixtures";
import { emitTauriEvent, mockTauriCommands } from "../test/mocks";
import { useWizardStore } from "../stores/wizardStore";
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
    mockTauriCommands({ run_atomizer: vi.fn(() => atomizer.promise) });

    const { result } = renderHook(() => useAtomizerPipeline("project-001"));

    await waitFor(() => expect(result.current.atomizeStarted).toBe(true));
    expect(result.current.stages.map((stage) => stage.status)).toEqual([
      "running",
      "pending",
      "pending",
      "pending",
    ]);

    await act(async () => {
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
      atomizer.resolve(createAtomizedPrd());
      await atomizer.promise;
    });

    await waitFor(() => expect(result.current.isDone).toBe(true));
    expect(result.current.stageMessage).toBe("Done. 2 stories generated.");
    expect(useWizardStore.getState().stories).toEqual(createAtomizedPrd().stories);
  });

  it("marks the active stage as errored when atomization fails", async () => {
    mockTauriCommands({
      run_atomizer: vi.fn(async () => Promise.reject(new Error("Atomizer failed"))),
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

  it("short-circuits to completed state when stories are already in the store", async () => {
    const runAtomizerCommand = vi.fn(async () => createAtomizedPrd());
    useWizardStore.setState({ stories: createAtomizedPrd().stories });
    mockTauriCommands({ run_atomizer: runAtomizerCommand });

    const { result } = renderHook(() => useAtomizerPipeline("project-001"));

    await waitFor(() => expect(result.current.isDone).toBe(true));
    expect(result.current.stageMessage).toBe("Loaded existing stories.");
    expect(result.current.stages.every((stage) => stage.status === "done")).toBe(true);
    expect(runAtomizerCommand).not.toHaveBeenCalled();
  });
});
