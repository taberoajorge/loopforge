import assert from "node:assert/strict";
import { browser, expect } from "@wdio/globals";
import { runDesktopHappyPath } from "../helpers/desktop-flow.ts";

describe("LoopForge desktop happy path", () => {
  it("completes the primary successful desktop journey in an isolated environment", async () => {
    const artifacts = await runDesktopHappyPath(
      "Desktop happy path",
      "Create a stable desktop happy path test that validates the wizard flow and monitor handoff.",
    );

    await expect(browser).toHaveTitle("LoopForge");
    assert.ok(artifacts.plan.includes("Create the project"), "plan.md should contain fixture plan content");
    assert.equal(artifacts.prd.stories.length, 1, "prd.json should contain one generated story");
    assert.ok(artifacts.prd.stories[0]?.passes, "generated story should pass after loop execution");
    assert.equal(artifacts.config.executeAgent, "cursor", "config.json should persist the default execute agent");
    assert.ok(artifacts.prompt.includes("Execution Prompt"), "prompt.md should be generated during launch");
    assert.ok(
      artifacts.outputLog.includes("fixture agent completed"),
      "agent output log should capture fixture execution",
    );
  });
});
