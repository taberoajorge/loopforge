import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useParams } from "react-router";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useWizardStore } from "../../stores/wizardStore";
import { createUserStory, createWizardConfig, createWizardProjectData } from "../../test/fixtures";
import { mockTauriCommands, type TauriCommandArgs } from "../../test/mocks";
import { renderRoute } from "../../test/renderRoute";
import { Launch } from "./Launch";

function resetLaunchStore() {
  useWizardStore.getState().reset();
  useWizardStore.setState({
    projectId: "project-010",
    projectData: createWizardProjectData(),
    config: createWizardConfig(),
    stories: [
      createUserStory({ id: "S-010", estimatedMinutes: 30 }),
      createUserStory({ id: "S-011", estimatedMinutes: 90 }),
    ],
  });
}

function MonitorRouteProbe() {
  const params = useParams();
  return <div data-testid="monitor-route">{params.id}</div>;
}

function createDeferred<TValue>() {
  let resolve!: (value: TValue) => void;
  let reject!: (error?: unknown) => void;
  const promise = new Promise<TValue>((nextResolve, nextReject) => {
    resolve = nextResolve;
    reject = nextReject;
  });
  return { promise, resolve, reject };
}

describe("Launch", () => {
  beforeEach(() => {
    resetLaunchStore();
  });

  it("renders readiness details and disables launch when required data is missing", () => {
    mockTauriCommands({
      finalize_draft: vi.fn(),
      start_loop: vi.fn(),
    });
    useWizardStore.setState({
      projectData: createWizardProjectData({ workingDirectory: "" }),
      stories: [],
      config: createWizardConfig({ executeAgent: "" }),
    });

    renderRoute([{ path: "/new/launch/:id", element: <Launch /> }], ["/new/launch/project-010"]);

    expect(screen.getByText("Readiness")).toBeInTheDocument();
    expect(screen.getByText("Needs attention")).toBeInTheDocument();
    expect(screen.getByText("Working directory is missing.")).toBeInTheDocument();
    expect(screen.getByText("Add at least one story before launching.")).toBeInTheDocument();
    expect(screen.getByText("Execution agent is missing.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Launch loop" })).toBeDisabled();
  });

  it("enables launch, shows the pending state, and starts the loop with config payload", async () => {
    const user = userEvent.setup();
    const deferredStart = createDeferred<string>();
    let finalizedProjectId: string | undefined;
    let startLoopArgs: TauriCommandArgs<"start_loop"> | undefined;
    mockTauriCommands({
      finalize_draft: vi.fn((args: TauriCommandArgs<"finalize_draft">) => {
        finalizedProjectId = args.projectId;
        return undefined;
      }),
      start_loop: vi.fn((args: TauriCommandArgs<"start_loop">) => {
        startLoopArgs = args;
        return deferredStart.promise;
      }),
    });

    renderRoute(
      [
        { path: "/new/launch/:id", element: <Launch /> },
        { path: "/monitor/:id", element: <MonitorRouteProbe /> },
      ],
      ["/new/launch/project-010"],
    );

    expect(screen.getByText("Ready")).toBeInTheDocument();
    expect(screen.getByText("All required launch inputs are present.")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Launch loop" }));

    expect(screen.getByRole("button", { name: "Initializing..." })).toBeDisabled();
    expect(finalizedProjectId).toBe("project-010");
    expect(startLoopArgs).toEqual({
      args: {
        projectId: "project-010",
        agent: "codex",
        model: "gpt-5.4",
        effort: "medium",
      },
    });

    deferredStart.resolve("session-010");

    expect(await screen.findByTestId("monitor-route")).toHaveTextContent("project-010");
    expect(useWizardStore.getState().projectId).toBeNull();
  });

  it("renders the launch failure path without navigating", async () => {
    const user = userEvent.setup();
    mockTauriCommands({
      finalize_draft: vi.fn(),
      start_loop: vi.fn(async () => {
        throw new Error("Loop start failed");
      }),
    });

    renderRoute([{ path: "/new/launch/:id", element: <Launch /> }], ["/new/launch/project-010"]);

    await user.click(screen.getByRole("button", { name: "Launch loop" }));

    expect(await screen.findByText("Launch failed: Loop start failed")).toBeInTheDocument();
    expect(screen.queryByTestId("monitor-route")).not.toBeInTheDocument();
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Launch loop" })).toBeEnabled();
    });
  });
});
