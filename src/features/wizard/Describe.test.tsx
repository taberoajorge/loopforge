import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useParams } from "react-router";
import {
  createAgentCapabilities,
  createAgentInfo,
  createProjectRecord,
  createWizardProjectData,
} from "../../test/fixtures";
import {
  invokeMock,
  mockDialogSelection,
  mockTauriCommands,
  openDialogMock,
  type TauriCommandArgs,
} from "../../test/mocks";
import { renderRoute } from "../../test/renderRoute";
import { useAgentStore } from "../../stores/agentStore";
import { useWizardStore } from "../../stores/wizardStore";
import { Describe } from "./Describe";

function resetDescribeStores() {
  useAgentStore.setState({ agents: [], detecting: false });
  useWizardStore.getState().reset();
}

function PlanRouteProbe() {
  const params = useParams();
  return <div data-testid="plan-route">{params.id}</div>;
}

describe("Describe", () => {
  beforeEach(() => {
    resetDescribeStores();
    useWizardStore.setState({
      projectData: createWizardProjectData({
        name: "",
        description: "",
        workingDirectory: "",
        planAgent: "codex",
      }),
    });
  });

  it("renders the form, validates required fields, and marks plan output stale on description edits", async () => {
    const user = userEvent.setup();
    mockTauriCommands({
      list_connections: [],
      detect_agents: [createAgentInfo()],
      get_agent_capabilities: createAgentCapabilities(),
      create_project: createProjectRecord(),
      save_draft: undefined,
    });
    useWizardStore.setState({ planContent: "Existing plan output" });

    renderRoute(
      [
        { path: "/new/describe", element: <Describe /> },
        { path: "/new/plan/:id", element: <PlanRouteProbe /> },
      ],
      ["/new/describe"],
    );

    expect(await screen.findByTestId("describe-form")).toBeInTheDocument();
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("detect_agents");
      expect(invokeMock).toHaveBeenCalledWith("get_agent_capabilities", { agent: "codex" });
    });

    await user.click(screen.getByTestId("describe-next-button"));

    expect(await screen.findByText("Project name is required")).toBeInTheDocument();
    expect(screen.getByText("Feature description is required")).toBeInTheDocument();
    expect(screen.getByText("Working directory is required")).toBeInTheDocument();

    await user.type(screen.getByLabelText("Functional Specification"), "Protect the first wizard step.");

    await waitFor(() => {
      expect(useWizardStore.getState().staleFromStep).toBe(2);
    });
    expect(screen.queryByText("Feature description is required")).not.toBeInTheDocument();
  });

  it("persists the draft and advances to plan on a valid submit", async () => {
    const user = userEvent.setup();
    const createdProject = createProjectRecord({
      id: "project-007",
      name: "LoopForge Refactor",
      description: "Cover the first wizard flow",
      workingDirectory: "/work/loopforge",
    });
    const createProjectCommand = vi.fn(() => createdProject);
    let savedDraftArgs: TauriCommandArgs<"save_draft"> | undefined;
    const saveDraftCommand = vi.fn((args: TauriCommandArgs<"save_draft">) => {
      savedDraftArgs = args;
    });
    mockTauriCommands({
      list_connections: [],
      detect_agents: [createAgentInfo()],
      get_agent_capabilities: createAgentCapabilities(),
      create_project: createProjectCommand,
      save_draft: saveDraftCommand,
    });

    renderRoute(
      [
        { path: "/new/describe", element: <Describe /> },
        { path: "/new/plan/:id", element: <PlanRouteProbe /> },
      ],
      ["/new/describe"],
    );

    await screen.findByTestId("describe-form");
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("detect_agents");
    });

    await user.type(screen.getByLabelText("Project Name"), "LoopForge Refactor");
    await user.type(screen.getByPlaceholderText("/absolute/path/to/project"), "/work/loopforge");
    await user.type(
      screen.getByLabelText("Functional Specification"),
      "Cover the project entry path and the describe step.",
    );
    await user.click(screen.getByTestId("describe-next-button"));

    expect(await screen.findByTestId("plan-route")).toHaveTextContent(createdProject.id);
    expect(createProjectCommand).toHaveBeenCalledWith({
      name: "LoopForge Refactor",
      description: "Cover the project entry path and the describe step.",
      workingDirectory: "/work/loopforge",
      wizardStep: "describe",
    });
    expect(saveDraftCommand).toHaveBeenCalledTimes(1);
    expect(savedDraftArgs).toBeDefined();
    const savedDraft = savedDraftArgs as TauriCommandArgs<"save_draft">;
    expect(savedDraft.projectId).toBe(createdProject.id);
    expect(JSON.parse(savedDraft.draftJson)).toMatchObject({
      projectId: createdProject.id,
      currentStep: "plan",
      describe: {
        name: "LoopForge Refactor",
        description: "Cover the project entry path and the describe step.",
        workingDirectory: "/work/loopforge",
        planAgent: "codex",
      },
    });
    expect(useWizardStore.getState().projectId).toBe(createdProject.id);
    expect(useWizardStore.getState().currentStep).toBe(2);
    expect(useWizardStore.getState().projectData.description).toBe(
      "Cover the project entry path and the describe step.",
    );
  });

  it("fills the working directory from the mocked desktop dialog", async () => {
    const user = userEvent.setup();
    mockDialogSelection("/work/from-dialog");
    mockTauriCommands({
      list_connections: [],
      detect_agents: [createAgentInfo()],
      get_agent_capabilities: createAgentCapabilities(),
    });

    renderRoute([{ path: "/new/describe", element: <Describe /> }], ["/new/describe"]);

    await screen.findByTestId("describe-form");
    await user.click(screen.getByTestId("describe-browse-directory"));

    expect(openDialogMock).toHaveBeenCalledWith({ directory: true, multiple: false });
    expect(screen.getByDisplayValue("/work/from-dialog")).toBeInTheDocument();
  });
});
