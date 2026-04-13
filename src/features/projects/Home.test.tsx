import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useParams } from "react-router";
import { beforeEach, describe, expect, it } from "vitest";
import { useProjectStore } from "../../stores/projectStore";
import { createProject } from "../../test/fixtures";
import { invokeMock, mockTauriCommand } from "../../test/mocks";
import { renderRoute } from "../../test/renderRoute";
import { Home } from "./Home";

function resetProjectStore() {
  useProjectStore.setState({ projects: [], loading: false });
}

function ResumeRouteProbe() {
  const params = useParams();
  return <div data-testid="resume-route">{params.id}</div>;
}

describe("Home", () => {
  beforeEach(() => {
    resetProjectStore();
  });

  it("loads the empty home state and starts a new project", async () => {
    const user = userEvent.setup();
    mockTauriCommand("check_system_readiness", {
      agents: [{ name: "codex", binary: "codex", version: "0.1.0", available: true }],
      gitAvailable: true,
      shellAvailable: true,
      platform: "test",
    });
    mockTauriCommand("list_projects_enriched", []);

    renderRoute(
      [
        { path: "/", element: <Home /> },
        { path: "/new/describe", element: <div data-testid="describe-route" /> },
      ],
      ["/"],
    );

    expect(await screen.findByTestId("home-page")).toBeInTheDocument();
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("list_projects_enriched");
    });

    await user.click(screen.getByTestId("home-start-project-button"));

    expect(await screen.findByTestId("describe-route")).toBeInTheDocument();
  });

  it("renders draft projects from mocked IPC data and resumes the selected draft", async () => {
    const user = userEvent.setup();
    mockTauriCommand("check_system_readiness", {
      agents: [{ name: "codex", binary: "codex", version: "0.1.0", available: true }],
      gitAvailable: true,
      shellAvailable: true,
      platform: "test",
    });
    const draftProject = createProject({
      id: "draft-007",
      name: "Resume flow",
      status: "draft",
      wizardStep: "plan",
    });
    mockTauriCommand("list_projects_enriched", [draftProject]);

    renderRoute(
      [
        { path: "/", element: <Home /> },
        { path: "/new/plan/:id", element: <ResumeRouteProbe /> },
      ],
      ["/"],
    );

    expect(await screen.findByTestId(`draft-card-${draftProject.id}`)).toBeInTheDocument();

    await user.click(screen.getByTestId(`draft-resume-${draftProject.id}`));

    expect(await screen.findByTestId("resume-route")).toHaveTextContent(draftProject.id);
  });
});
