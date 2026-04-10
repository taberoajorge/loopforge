import assert from "node:assert/strict";
import { access } from "node:fs/promises";
import path from "node:path";
import { browser, expect } from "@wdio/globals";
import { projectDir, seedProjectFixture } from "../helpers/project-fixtures.ts";

const projectId = "atomizer-error-001";

async function pathExists(targetPath: string) {
  try {
    await access(targetPath);
    return true;
  } catch {
    return false;
  }
}

describe("LoopForge atomize desktop flow: atomizer error", () => {
  it("shows the atomizer error state and stays out of execution", async () => {
    await seedProjectFixture({
      id: projectId,
      name: "Atomizer error fixture",
      description: "Exercise the atomizer failure branch before execution begins.",
      wizardStep: "atomize",
      draft: {
        version: 1,
        projectId,
        currentStep: "atomize",
        describe: {
          name: "Atomizer error fixture",
          description: "Exercise the atomizer failure branch before execution begins.",
          workingDirectory: process.env.LOOPFORGE_E2E_WORKSPACE_DIR,
          planAgent: "claude",
        },
        plan: { completed: true },
        atomize: { storiesCount: 0 },
        configure: {},
      },
      plan: "# Atomizer Error Fixture\n\n1. Force a deterministic atomizer failure.\n",
    });

    await browser.refresh();
    await (await $(`[data-testid="draft-resume-${projectId}"]`)).waitForDisplayed({ timeout: 30000 });
    await (await $(`[data-testid="draft-resume-${projectId}"]`)).click();

    await expect($("*=JSON parse error at stage chunk")).toBeDisplayed();
    await expect($('[data-testid="atomize-story-list-status"]')).toHaveText(
      "Atomization failed. Add stories manually or go back.",
    );
    await expect($("button=Proceed to validation")).toBeDisabled();
    assert.ok((await browser.getUrl()).includes(`/new/atomize/${projectId}`));
    assert.equal(await pathExists(path.join(projectDir(projectId), "prd.json")), false);
  });
});
