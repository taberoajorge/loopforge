import { act, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useWizardStore } from "../../stores/wizardStore";
import {
  createAtomizedPrd,
  createAtomizeProgress,
  createPrd,
  createWizardProjectData,
} from "../../test/fixtures";
import { emitTauriEvent, mockTauriCommands } from "../../test/mocks";
import { renderRoute } from "../../test/renderRoute";
import { Atomize } from "./Atomize";

function createDeferred<TValue>() {
  let resolve!: (value: TValue) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<TValue>((nextResolve, nextReject) => {
    resolve = nextResolve;
    reject = nextReject;
  });
  return { promise, resolve, reject };
}

function resetAtomizeStore() {
  useWizardStore.getState().reset();
  useWizardStore.setState({
    projectId: "project-001",
    projectData: createWizardProjectData(),
  });
}

function renderAtomizeRoute() {
  return renderRoute(
    [{ path: "/new/atomize/:id", element: <Atomize /> }],
    ["/new/atomize/project-001"],
  );
}

describe("Atomize", () => {
  beforeEach(() => {
    resetAtomizeStore();
  });

  it("renders loading state, progress updates, and completed stories", async () => {
    const atomizer = createDeferred<ReturnType<typeof createAtomizedPrd>>();
    const runAtomizerCommand = vi.fn(() => atomizer.promise);
    mockTauriCommands({ run_atomizer: runAtomizerCommand });

    renderAtomizeRoute();

    await waitFor(() => expect(runAtomizerCommand).toHaveBeenCalledTimes(1));
    expect(screen.getByTestId("atomize-story-list-status")).toHaveTextContent(
      "Atomizing plan into stories...",
    );
    expect(screen.getByRole("button", { name: "Add story" })).toBeDisabled();

    await act(async () => {
      await emitTauriEvent(
        "atomization-progress",
        createAtomizeProgress({
          stage: 2,
          stageName: "Chunk",
          message: "Chunking plan into sections",
        }),
      );
    });

    expect(screen.getByText(/Chunking plan into sections/)).toBeInTheDocument();

    await act(async () => {
      atomizer.resolve(createAtomizedPrd());
      await atomizer.promise;
    });

    expect(await screen.findByTestId("atomize-story-list")).toBeInTheDocument();
    expect(screen.getByTestId("atomize-story-S-005")).toBeInTheDocument();
    expect(screen.getByTestId("atomize-story-S-006")).toBeInTheDocument();
    expect(screen.getByText("Done. 2 stories generated.")).toBeInTheDocument();
    expect(screen.getByText("Atomization Complete · 2 Stories · 1.7h")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Add story" })).toBeEnabled();
  });

  it("loads persisted stories without rerunning the atomizer", async () => {
    const runAtomizerCommand = vi.fn(async () => createAtomizedPrd());
    useWizardStore.setState({ stories: createAtomizedPrd().stories });
    mockTauriCommands({ run_atomizer: runAtomizerCommand });

    renderAtomizeRoute();

    await waitFor(() => {
      expect(screen.getByText("Loaded existing stories.")).toBeInTheDocument();
    });
    expect(screen.getByTestId("atomize-story-S-005")).toBeInTheDocument();
    expect(screen.getByTestId("atomize-story-S-006")).toBeInTheDocument();
    expect(runAtomizerCommand).not.toHaveBeenCalled();
  });

  it("shows an atomizer error state without enabling validation", async () => {
    mockTauriCommands({
      run_atomizer: vi.fn(async () => Promise.reject(new Error("JSON parse error at stage chunk"))),
    });

    renderAtomizeRoute();

    expect(await screen.findByText("JSON parse error at stage chunk")).toBeInTheDocument();
    expect(screen.getByTestId("atomize-story-list-status")).toHaveTextContent(
      "Atomization failed. Add stories manually or go back.",
    );
    expect(screen.getByRole("button", { name: "Add story" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Proceed to validation" })).toBeDisabled();
  });

  it("shows an empty result state when atomization succeeds with no stories", async () => {
    mockTauriCommands({
      run_atomizer: vi.fn(async () => createPrd({ stories: [], totalEstimatedMinutes: 0 })),
    });

    renderAtomizeRoute();

    expect(await screen.findByText("Done. 0 stories generated.")).toBeInTheDocument();
    expect(screen.getByTestId("atomize-story-list-status")).toHaveTextContent(
      "Atomization completed with no stories. Add a story manually or go back.",
    );
    expect(screen.getByText("Atomization Complete · 0 Stories")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Add story" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Proceed to validation" })).toBeDisabled();
  });
});
