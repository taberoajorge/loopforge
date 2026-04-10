import { browser, expect } from "@wdio/globals";
import {
  projectArtifactExists,
  projectRecordExists,
  seedProjectFixture,
} from "../helpers/project-fixtures.ts";

const projectId = "draft-discard-001";

describe("LoopForge planning entry branches: draft discard", () => {
  it("returns to a clean start state after discarding a seeded draft", async () => {
    await seedProjectFixture({
      id: projectId,
      name: "Discardable draft",
      description: "Return home cleanly after discard.",
      wizardStep: "plan",
      draft: {
        version: 1,
        projectId,
        currentStep: "plan",
        describe: {
          name: "Discardable draft",
          description: "Return home cleanly after discard.",
          workingDirectory: process.env.LOOPFORGE_E2E_WORKSPACE_DIR,
          planAgent: "claude",
        },
        plan: { completed: false },
        atomize: { storiesCount: 0 },
        configure: {},
      },
      plan: "# Draft plan fixture",
    });

    await browser.refresh();
    const draftCard = await $(`[data-testid="draft-card-${projectId}"]`);
    await draftCard.waitForDisplayed({ timeout: 30000 });
    await (await draftCard.$("button=Discard")).click();

    await browser.waitUntil(
      async () => !(await projectRecordExists(projectId)) && !(await projectArtifactExists(projectId)),
      { timeout: 30000, interval: 250, timeoutMsg: "expected draft artifacts to be removed" },
    );

    await expect($("*=No projects yet")).toBeDisplayed();
    await expect($('[data-testid="home-start-project-button"]')).toBeDisplayed();
  });
});
