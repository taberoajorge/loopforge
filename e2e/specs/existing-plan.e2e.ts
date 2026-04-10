import assert from "node:assert/strict";
import { browser, expect } from "@wdio/globals";
import { readProjectPlan, seedProjectFixture } from "../helpers/project-fixtures.ts";

const projectId = "existing-plan-001";
const seededPlan = [
  "# Existing Plan Fixture",
  "",
  "1. Review seeded context.",
  "2. Keep the existing plan intact.",
].join("\n");

describe("LoopForge planning entry branches: existing plan", () => {
  it("opens a seeded plan without regenerating it", async () => {
    await seedProjectFixture({
      id: projectId,
      name: "Existing plan fixture",
      description: "Resume from a seeded plan artifact.",
      wizardStep: "plan",
      draft: {
        version: 1,
        projectId,
        currentStep: "plan",
        describe: {
          name: "Existing plan fixture",
          description: "Resume from a seeded plan artifact.",
          workingDirectory: process.env.LOOPFORGE_E2E_WORKSPACE_DIR,
          planAgent: "claude",
        },
        plan: { completed: true },
        atomize: { storiesCount: 0 },
        configure: {},
      },
      plan: seededPlan,
    });

    await browser.refresh();
    await (await $(`[data-testid="draft-resume-${projectId}"]`)).waitForDisplayed({ timeout: 30000 });
    await (await $(`[data-testid="draft-resume-${projectId}"]`)).click();

    await expect($("*=An existing plan was found.")).toBeDisplayed();
    await expect($("*=Review seeded context.")).toBeDisplayed();
    await (await $('[data-testid="plan-stream-use-existing-button"]')).click();

    await expect($("button=Next")).toBeEnabled();
    await expect($('[data-testid="plan-stream-send-button"]')).toHaveText("Re-plan");
    assert.equal(await readProjectPlan(projectId), seededPlan);
  });
});
