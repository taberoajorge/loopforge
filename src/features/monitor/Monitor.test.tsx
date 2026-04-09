import { act, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { createIterationRow, createProjectScopedEventPayload, createProjectSnapshot } from "../../test/fixtures";
import type { IterationStory } from "../../lib/tauri";
import { emitTauriEvent, mockTauriCommands } from "../../test/mocks";
import { renderRoute } from "../../test/renderRoute";
import { useProjectStore } from "../../stores/projectStore";
import { Monitor } from "./Monitor";

vi.mock("./OutputTab", () => ({ OutputTab: () => <div data-testid="output-tab" /> }));
vi.mock("./AskTab", () => ({ AskTab: () => <div data-testid="ask-tab" /> }));
vi.mock("./components/ExecutionProfilePanel", () => ({
  ExecutionProfilePanel: () => <div data-testid="config-panel" />,
}));
vi.mock("../../components/EphemeralOverlay", () => ({
  EphemeralOverlay: () => <div data-testid="ephemeral-overlay" />,
}));

const STORIES: IterationStory[] = [
  { id: "S-005", title: "Stabilize plan flow", status: "current", attempts: 1 },
  { id: "S-006", title: "Render monitor updates", status: "pending", attempts: 0 },
];

function renderMonitor() {
  return renderRoute([{ path: "/monitor/:id", element: <Monitor /> }], ["/monitor/project-001"]);
}

describe("Monitor", () => {
  beforeEach(() => {
    useProjectStore.setState({ projects: [], loading: false });
  });

  it("renders live session state and responds to iteration events", async () => {
    mockTauriCommands({
      get_project_snapshot: createProjectSnapshot(),
      get_project_stories: STORIES,
      get_iteration_history: [],
    });

    renderMonitor();

    expect(await screen.findByText("SESSION MONITOR: LoopForge")).toBeInTheDocument();
    expect(screen.getAllByText("Running")).toHaveLength(2);
    expect(screen.getByText("1/3 stories")).toBeInTheDocument();
    expect(screen.getByText("33% complete")).toBeInTheDocument();

    await waitFor(() => expect(screen.getByText("S-006")).toBeInTheDocument());

    await act(async () => {
      await emitTauriEvent(
        "loop:iteration-started",
        createProjectScopedEventPayload({
          storyId: "S-006",
          agent: "codex",
          timestamp: "2026-04-09T11:00:00.000Z",
        }),
      );
    });

    const progressTable = screen.getByRole("table");
    const pendingRow = within(progressTable).getByText("S-005").closest("tr");
    const currentRow = within(progressTable).getByText("S-006").closest("tr");
    expect(pendingRow).not.toBeNull();
    expect(currentRow).not.toBeNull();
    expect(within(pendingRow as HTMLElement).getByText("pending")).toBeInTheDocument();
    expect(within(currentRow as HTMLElement).getByText("current")).toBeInTheDocument();

    await userEvent.setup().click(screen.getByRole("tab", { name: "Activity" }));

    await act(async () => {
      await emitTauriEvent(
        "loop:iteration-completed",
        createProjectScopedEventPayload({
          storyId: "S-006",
          agent: "codex",
          result: "success",
          durationSecs: 34,
          timestamp: "2026-04-09T11:01:00.000Z",
        }),
      );
    });

    expect(await screen.findByText("success")).toBeInTheDocument();
    expect(screen.getAllByText("S-006").length).toBeGreaterThan(0);
    expect(screen.getByText("codex")).toBeInTheDocument();
  });

  it("runs pause, resume, and stop actions against mocked commands", async () => {
    const pauseProject = vi.fn(async () => {});
    const resumeProject = vi.fn(async () => {});
    const stopLoop = vi.fn(async () => {});
    const snapshotResponses = [
      createProjectSnapshot(),
      createProjectSnapshot({ status: "paused" }),
      createProjectSnapshot({ status: "running" }),
      createProjectSnapshot({ status: "failed" }),
    ];
    const listProjects = vi.fn(async () => []);

    mockTauriCommands({
      get_project_snapshot: vi.fn(async () => snapshotResponses.shift() ?? createProjectSnapshot({ status: "failed" })),
      get_project_stories: STORIES,
      get_iteration_history: [createIterationRow()],
      pause_project: pauseProject,
      resume_project: resumeProject,
      stop_loop: stopLoop,
      list_projects_enriched: listProjects,
    });

    const user = userEvent.setup();
    renderMonitor();

    expect(await screen.findByRole("button", { name: "Pause" })).toBeEnabled();

    await user.click(screen.getByRole("button", { name: "Pause" }));
    await waitFor(() => expect(pauseProject).toHaveBeenCalledWith({ projectId: "project-001" }));
    expect(await screen.findByRole("button", { name: "Resume" })).toBeEnabled();

    await user.click(screen.getByRole("button", { name: "Resume" }));
    await waitFor(() => expect(resumeProject).toHaveBeenCalledWith({ projectId: "project-001" }));
    expect(await screen.findByRole("button", { name: "Pause" })).toBeEnabled();

    await user.click(screen.getByRole("button", { name: "Stop" }));
    await waitFor(() => expect(stopLoop).toHaveBeenCalledWith({ projectId: "project-001" }));
    expect(listProjects).toHaveBeenCalledTimes(3);
  });
});
