import { beforeEach, describe, expect, it } from "vitest";
import { createUserStory } from "../test/fixtures";
import { useWizardStore } from "./wizardStore";

describe("wizardStore", () => {
  beforeEach(() => {
    useWizardStore.getState().reset();
  });

  it("merges project and config updates without dropping required defaults", () => {
    useWizardStore.getState().setProjectData({
      name: "LoopForge",
      description: "Protect the wizard store",
      workingDirectory: "/work/loopforge",
    });
    useWizardStore.getState().setConfig({
      executeAgent: "codex",
      maxIterations: 80,
    });

    expect(useWizardStore.getState().projectData).toEqual({
      name: "LoopForge",
      description: "Protect the wizard store",
      workingDirectory: "/work/loopforge",
      planAgent: "claude",
      planModel: null,
      planEffort: null,
    });
    expect(useWizardStore.getState().config).toEqual({
      executeAgent: "codex",
      executeModel: null,
      executeEffort: null,
      fallbackChain: ["claude"],
      gutterThreshold: 3,
      maxIterations: 80,
      cooldownSeconds: 5,
      testCommand: "",
      maxVerificationRetries: 3,
      scmProvider: "auto",
      reviewPollingInterval: 60,
      reviewTimeout: 600,
    });
  });

  it("resets all slices back to their initial state", () => {
    useWizardStore.setState({
      projectId: "project-001",
      currentStep: 4,
      highestStep: 5,
      planComplete: true,
      stories: [createUserStory()],
    });

    useWizardStore.getState().reset();

    expect(useWizardStore.getState().projectId).toBeNull();
    expect(useWizardStore.getState().currentStep).toBe(1);
    expect(useWizardStore.getState().highestStep).toBe(1);
    expect(useWizardStore.getState().planComplete).toBe(false);
    expect(useWizardStore.getState().stories).toEqual([]);
    expect(useWizardStore.getState().projectData.planAgent).toBe("claude");
    expect(useWizardStore.getState().config.executeAgent).toBe("cursor");
  });
});
