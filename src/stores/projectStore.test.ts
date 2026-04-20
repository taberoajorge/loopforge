import { beforeEach, describe, expect, it, vi } from "vitest";
import { createProject, createProjectSnapshot } from "../test/fixtures";
import { mockTauriCommand } from "../test/mocks";
import { useProjectStore } from "./projectStore";

describe("projectStore", () => {
  beforeEach(() => {
    useProjectStore.setState({
      projects: [],
      grouped: { active: [], drafts: [], finished: [], archived: [] },
      loading: false,
    });
  });

  it("fetches projects from mocked IPC and clears loading state", async () => {
    const activeProject = createProject({ id: "project-002", status: "active" });
    mockTauriCommand("list_projects_grouped", {
      active: [activeProject],
      drafts: [],
      finished: [],
      archived: [],
    });

    await useProjectStore.getState().fetchProjects();

    expect(useProjectStore.getState().loading).toBe(false);
    expect(useProjectStore.getState().projects).toEqual([activeProject]);
  });

  it("keeps loading false when the project list request fails", async () => {
    mockTauriCommand(
      "list_projects_grouped",
      vi.fn(async () => {
        throw new Error("IPC failed");
      }),
    );

    await useProjectStore.getState().fetchProjects();

    expect(useProjectStore.getState().loading).toBe(false);
    expect(useProjectStore.getState().projects).toEqual([]);
  });

  it("applies snapshot events to project cache", () => {
    const running = createProjectSnapshot({
      project: { id: "project-001", name: "First project" },
      status: "running",
    });
    const paused = createProjectSnapshot({
      project: { id: "project-001", name: "First project paused" },
      status: "paused",
    });
    const completed = createProjectSnapshot({
      project: { id: "project-002", name: "Second project" },
      status: "completed",
    });
    useProjectStore.getState().applySnapshot(running);
    useProjectStore.getState().applySnapshot(paused);
    useProjectStore.getState().applySnapshot(completed);
    expect(useProjectStore.getState().projects).toHaveLength(2);
    expect(useProjectStore.getState().projects[0].name).toBe("First project paused");
    expect(useProjectStore.getState().grouped.active).toHaveLength(1);
    expect(useProjectStore.getState().grouped.finished).toHaveLength(1);
  });
});
