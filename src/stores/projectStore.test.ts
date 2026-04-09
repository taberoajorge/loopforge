import { beforeEach, describe, expect, it, vi } from "vitest";
import { createProject } from "../test/fixtures";
import { mockTauriCommand } from "../test/mocks";
import { useProjectStore } from "./projectStore";

describe("projectStore", () => {
  beforeEach(() => {
    useProjectStore.setState({ projects: [], loading: false });
  });

  it("fetches projects from mocked IPC and clears loading state", async () => {
    const activeProject = createProject({ id: "project-002", status: "active" });
    mockTauriCommand("list_projects_enriched", [activeProject]);

    await useProjectStore.getState().fetchProjects();

    expect(useProjectStore.getState().loading).toBe(false);
    expect(useProjectStore.getState().projects).toEqual([activeProject]);
  });

  it("keeps loading false when the project list request fails", async () => {
    mockTauriCommand("list_projects_enriched", vi.fn(async () => {
      throw new Error("IPC failed");
    }));

    await useProjectStore.getState().fetchProjects();

    expect(useProjectStore.getState().loading).toBe(false);
    expect(useProjectStore.getState().projects).toEqual([]);
  });

  it("upserts projects by id without duplicating entries", () => {
    const initial = createProject({ id: "project-001", name: "Initial name" });
    const updated = createProject({ id: "project-001", name: "Updated name", status: "paused" });
    const inserted = createProject({ id: "project-002", name: "Second project" });

    useProjectStore.setState({ projects: [initial], loading: false });

    useProjectStore.getState().upsertProject(updated);
    useProjectStore.getState().upsertProject(inserted);

    expect(useProjectStore.getState().projects).toEqual([updated, inserted]);
  });
});
