import assert from "node:assert/strict";
import { browser, expect } from "@wdio/globals";
import { seedProjectFixture } from "../helpers/project-fixtures.ts";

const projectId = "monitor-disabled-001";
const projectName = "Monitor disabled fixture";
const description = "Exercise disabled monitor affordances for inactive execution states.";

function buildConfig() {
  return {
    executeAgent: "codex",
    executeModel: "fixture-gpt-5.4",
    executeEffort: "medium",
    fallbackChain: ["claude"],
    gutterThreshold: 3,
    maxIterations: 50,
    cooldownSeconds: 5,
    testCommand: "",
    maxVerificationRetries: 3,
    scmProvider: "auto",
    reviewPollingInterval: 60,
    reviewTimeout: 600,
  };
}

function buildPrd() {
  return {
    projectName,
    stories: [
      {
        id: "S-027",
        title: "Cover monitor disabled states",
        description: "Ensure monitor actions stay unavailable when prerequisites are missing.",
        acceptanceCriteria: ["Disabled controls remain unavailable"],
        passes: false,
      },
    ],
  };
}

describe("LoopForge execution flow: monitor disabled states", () => {
  it("renders disabled monitor controls for an inactive failed session", async () => {
    await seedProjectFixture({
      id: projectId,
      name: projectName,
      description,
      status: "failed",
      wizardStep: "launch",
      draft: {
        version: 1,
        projectId,
        currentStep: "launch",
        describe: {
          name: projectName,
          description,
          workingDirectory: process.env.LOOPFORGE_E2E_WORKSPACE_DIR,
          planAgent: "claude",
        },
        plan: { completed: true },
        atomize: { storiesCount: 1 },
        configure: buildConfig(),
      },
      plan: "# Monitor Disabled Fixture\n\n1. Keep monitor actions unavailable.\n",
      prd: JSON.stringify(buildPrd(), null, 2),
      config: JSON.stringify(buildConfig(), null, 2),
    });

    await browser.refresh();
    await (await $(`*=${projectName}`)).waitForDisplayed({ timeout: 30000 });
    await (await $(`*=${projectName}`)).click();

    await expect($(`h2*=SESSION MONITOR: ${projectName}`)).toBeDisplayed();
    await expect($("button=Pause")).toBeDisabled();
    await expect($("button=Stop")).toBeDisabled();
    const askTab = await $("button=Ask");
    await expect(askTab).toBeDisabled();
    assert.equal(await askTab.getAttribute("aria-selected"), "false");

    await (await $("button=Config")).click();
    await expect($('[data-testid="execution-profile-save-button"]')).toBeDisabled();
    await expect($("*=Pause loop to edit execution profile, then resume to apply.")).toBeDisplayed();
  });
});
