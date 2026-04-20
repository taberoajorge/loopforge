import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useWizardStore } from "../stores/wizardStore";
import { createAtomizedPrd, createWizardResumeState } from "../test/fixtures";
import { mockTauriCommands } from "../test/mocks";
import { useWizardHydration } from "./useWizardHydration";

function resetWizardStore() {
  useWizardStore.getState().reset();
}

describe("useWizardHydration", () => {
  beforeEach(() => {
    resetWizardStore();
  });

  it("hydrates draft and prd data while preserving omitted defaults", async () => {
    mockTauriCommands({
      resume_wizard: createWizardResumeState({
        hasPlan: true,
        hasPrd: true,
        project: {
          id: "project-001",
          name: "Resume Name",
          description: "Resume Description",
          workingDirectory: "/resume/path",
        },
      }),
      load_draft: JSON.stringify({
        describe: {
          name: "Draft Name",
          description: "Draft Description",
          workingDirectory: "/draft/path",
          planModel: "gpt-5.4",
        },
        plan: { completed: true },
        configure: { executeAgent: "codex", maxIterations: 80 },
      }),
      load_existing_prd: createAtomizedPrd(),
    });

    const { result } = renderHook(() => useWizardHydration("project-001", null, ""));

    await waitFor(() => expect(result.current.hydrating).toBe(false));

    expect(useWizardStore.getState().projectId).toBe("project-001");
    expect(useWizardStore.getState().projectData).toEqual({
      name: "Draft Name",
      description: "Draft Description",
      workingDirectory: "/draft/path",
      planAgent: "claude",
      planModel: "gpt-5.4",
      planEffort: null,
    });
    expect(useWizardStore.getState().planComplete).toBe(true);
    expect(useWizardStore.getState().stories).toEqual(createAtomizedPrd().stories);
    expect(useWizardStore.getState().config.executeAgent).toBe("codex");
    expect(useWizardStore.getState().config.maxIterations).toBe(80);
    expect(useWizardStore.getState().config.reviewTimeout).toBe(600);
  });

  it("falls back to resume data when draft parsing fails", async () => {
    const loadExistingPrdCommand = vi.fn(async () => createAtomizedPrd());
    mockTauriCommands({
      resume_wizard: createWizardResumeState({
        hasPlan: false,
        hasPrd: false,
        project: {
          id: "project-002",
          name: "Resume Only",
          description: "Recovered from storage",
          workingDirectory: "/resume-only",
        },
      }),
      load_draft: "{bad json",
      load_existing_prd: loadExistingPrdCommand,
    });

    const { result } = renderHook(() => useWizardHydration("project-002", null, ""));

    await waitFor(() => expect(result.current.hydrating).toBe(false));

    expect(useWizardStore.getState().projectId).toBe("project-002");
    expect(useWizardStore.getState().projectData).toEqual({
      name: "Resume Only",
      description: "Recovered from storage",
      workingDirectory: "/resume-only",
      planAgent: "claude",
      planModel: null,
      planEffort: null,
    });
    expect(useWizardStore.getState().stories).toEqual([]);
    expect(useWizardStore.getState().planComplete).toBe(false);
    expect(loadExistingPrdCommand).not.toHaveBeenCalled();
  });
});
