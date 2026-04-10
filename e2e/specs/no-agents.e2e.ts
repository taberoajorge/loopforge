import assert from "node:assert/strict";
import { browser, expect } from "@wdio/globals";

function readEnv(name: string): string {
  const value = process.env[name];
  assert.ok(value, `Missing ${name}`);
  return value;
}

describe("LoopForge planning entry branches: no agents", () => {
  it("blocks planning when no agents are available", async () => {
    await expect(browser).toHaveTitle("LoopForge");
    await (await $('[data-testid="home-start-project-button"]')).click();
    await (await $('[data-testid="describe-page"]')).waitForDisplayed({ timeout: 30000 });

    await (await $('input[placeholder="e.g. auth-refactor"]')).setValue("No agents fixture");
    await (await $('input[placeholder="/absolute/path/to/project"]')).setValue(
      readEnv("LOOPFORGE_E2E_WORKSPACE_DIR"),
    );
    await (await $('textarea[placeholder^="Describe what you want to build."]')).setValue(
      "Validate the blocked planning state when no agents are detected.",
    );
    await (await $('[data-testid="describe-next-button"]')).click();

    await expect($("span=0 detected")).toBeDisplayed();
    await expect($("*=No supported agent was detected.")).toBeDisplayed();
    await expect($("*=Not installed")).toBeDisplayed();
  });
});
