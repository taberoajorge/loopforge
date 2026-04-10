import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { browser, expect } from "@wdio/globals";
import { projectDir, seedProjectFixture } from "../helpers/project-fixtures.ts";

const projectId = "no-stories-001";

describe("LoopForge atomize desktop flow: no stories", () => {
  it("shows the empty atomize result state without advancing into execution", async () => {
    await seedProjectFixture({
      id: projectId,
      name: "No stories fixture",
      description: "Exercise the empty atomize result branch before execution begins.",
      wizardStep: "atomize",
      draft: {
        version: 1,
        projectId,
        currentStep: "atomize",
        describe: {
          name: "No stories fixture",
          description: "Exercise the empty atomize result branch before execution begins.",
          workingDirectory: process.env.LOOPFORGE_E2E_WORKSPACE_DIR,
          planAgent: "claude",
        },
        plan: { completed: true },
        atomize: { storiesCount: 0 },
        configure: {},
      },
      plan: "# No Stories Fixture\n\n1. Return a valid PRD with zero stories.\n",
    });

    await browser.refresh();
    await (await $(`[data-testid="draft-resume-${projectId}"]`)).waitForDisplayed({ timeout: 30000 });
    await (await $(`[data-testid="draft-resume-${projectId}"]`)).click();

    await expect($("*=Done. 0 stories generated.")).toBeDisplayed();
    await expect($('[data-testid="atomize-story-list-status"]')).toHaveText(
      "Atomization completed with no stories. Add a story manually or go back.",
    );
    await expect($("*=Atomization Complete · 0 Stories")).toBeDisplayed();
    await expect($("button=Proceed to validation")).toBeDisabled();
    assert.ok((await browser.getUrl()).includes(`/new/atomize/${projectId}`));

    const prd = JSON.parse(
      await readFile(path.join(projectDir(projectId), "prd.json"), "utf8"),
    ) as { stories: unknown[] };
    assert.equal(prd.stories.length, 0);
  });
});
