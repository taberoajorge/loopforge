import assert from "node:assert/strict";
import { browser, expect } from "@wdio/globals";
import { readProjectPlan, seedProjectFixture } from "../helpers/project-fixtures.ts";

const projectId = "re-plan-001";
const seededPlan = [
  "# Existing Plan Fixture",
  "",
  "1. Seeded plan should be replaced.",
].join("\n");

describe("LoopForge planning entry branches: re-plan", () => {
  it("restarts planning from a pre-existing plan state", async () => {
    await seedProjectFixture({
      id: projectId,
      name: "Re-plan fixture",
      description: "Regenerate a plan from seeded state.",
      wizardStep: "plan",
      draft: {
        version: 1,
        projectId,
        currentStep: "plan",
        describe: {
          name: "Re-plan fixture",
          description: "Regenerate a plan from seeded state.",
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
    await (await $('[data-testid="plan-stream-use-existing-button"]')).click();
    await (await $("button=Re-plan")).click();

    await browser.waitUntil(async () => {
      const nextPlan = await readProjectPlan(projectId);
      return nextPlan.includes("# Fixture Planning Summary");
    }, { timeout: 30000, interval: 250, timeoutMsg: "expected re-plan fixture output" });

    const nextPlan = await readProjectPlan(projectId);
    assert.ok(nextPlan.includes("# Fixture Planning Summary"));
    assert.ok(!nextPlan.includes("Seeded plan should be replaced."));
    await expect($("*=Fixture Planning Summary")).toBeDisplayed();
    await expect($("button=Next")).toBeEnabled();
  });
});
