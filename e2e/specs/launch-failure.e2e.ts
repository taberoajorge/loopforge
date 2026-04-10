import assert from "node:assert/strict";
import { browser, expect } from "@wdio/globals";
import { seedProjectFixture } from "../helpers/project-fixtures.ts";

const projectId = "launch-failure-001";
const projectName = "Launch failure fixture";
const description = "Exercise the launch error path before the monitor opens.";

function readEnv(name: string): string {
  const value = process.env[name];
  assert.ok(value, `Missing ${name}`);
  return value;
}

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
        title: "Cover launch failure",
        description: "Keep the launch screen responsive after a startup error.",
        acceptanceCriteria: ["Launch error is shown"],
        passes: false,
      },
    ],
  };
}

describe("LoopForge execution flow: launch failure", () => {
  it("shows a visible startup failure and leaves launch actions responsive", async () => {
    await seedProjectFixture({
      id: projectId,
      name: projectName,
      description,
      wizardStep: "launch",
      draft: {
        version: 1,
        projectId,
        currentStep: "launch",
        describe: {
          name: projectName,
          description,
          workingDirectory: readEnv("LOOPFORGE_E2E_WORKSPACE_DIR"),
          planAgent: "claude",
        },
        plan: { completed: true },
        atomize: { storiesCount: 1 },
        configure: buildConfig(),
      },
      plan: "# Launch Failure Fixture\n\n1. Fail before monitor navigation.\n",
      prd: JSON.stringify(buildPrd(), null, 2),
      config: "{broken",
    });

    await browser.refresh();
    await (await $(`[data-testid="draft-resume-${projectId}"]`)).waitForDisplayed({ timeout: 30000 });
    await (await $(`[data-testid="draft-resume-${projectId}"]`)).click();

    await expect($("h2=Launch review")).toBeDisplayed();
    await expect($("*=All required launch inputs are present.")).toBeDisplayed();

    const launchButton = await $("button=Launch loop");
    await launchButton.click();

    await expect($("*=Launch failed:")).toBeDisplayed();
    await expect($("*=JSON error:")).toBeDisplayed();
    await expect($("h2=Launch review")).toBeDisplayed();
    await expect(launchButton).toBeEnabled();
    await expect($("button=Back")).toBeEnabled();
    assert.ok((await browser.getUrl()).includes(`/new/launch/${projectId}`));
  });
});
