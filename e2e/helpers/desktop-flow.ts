import assert from "node:assert/strict";
import { readdir, readFile } from "node:fs/promises";
import path from "node:path";
import { browser, $, expect } from "@wdio/globals";

type HappyPathArtifacts = {
  projectId: string;
  projectDir: string;
  plan: string;
  prd: { stories: Array<{ id: string; passes: boolean }> };
  config: { executeAgent: string };
  prompt: string;
  outputLog: string;
};

function readEnv(name: string): string {
  const value = process.env[name];
  assert.ok(value, `Missing ${name}`);
  return value;
}

function appDataDir(): string {
  const homeDir = readEnv("LOOPFORGE_E2E_HOME_DIR");
  const xdgDataHome = readEnv("LOOPFORGE_E2E_XDG_DATA_HOME");
  if (process.platform === "darwin") {
    return path.join(homeDir, "Library", "Application Support", "com.loopforge.desktop");
  }
  if (process.platform === "win32") {
    return path.join(homeDir, "AppData", "Roaming", "com.loopforge.desktop");
  }
  return path.join(xdgDataHome, "com.loopforge.desktop");
}

function projectsDir(): string {
  return path.join(appDataDir(), "projects");
}

async function waitForButton(label: string) {
  const button = await $(`button=${label}`);
  await button.waitForDisplayed({ timeout: 30000 });
  return button;
}

async function waitForSingleProjectId() {
  await browser.waitUntil(async () => {
    try {
      const entries = await readdir(projectsDir(), { withFileTypes: true });
      return entries.filter((entry) => entry.isDirectory()).length === 1;
    } catch {
      return false;
    }
  }, { timeout: 30000, interval: 250, timeoutMsg: "expected one project directory" });
  const entries = await readdir(projectsDir(), { withFileTypes: true });
  const projectDir = entries.find((entry) => entry.isDirectory());
  assert.ok(projectDir, "expected project directory");
  return projectDir.name;
}

async function readJson<T>(filePath: string): Promise<T> {
  return JSON.parse(await readFile(filePath, "utf8")) as T;
}

async function readArtifacts(projectId: string): Promise<HappyPathArtifacts> {
  const projectDir = path.join(projectsDir(), projectId);
  const [plan, prd, config, prompt, outputLog] = await Promise.all([
    readFile(path.join(projectDir, "plan.md"), "utf8"),
    readJson<HappyPathArtifacts["prd"]>(path.join(projectDir, "prd.json")),
    readJson<HappyPathArtifacts["config"]>(path.join(projectDir, "config.json")),
    readFile(path.join(projectDir, "prompt.md"), "utf8"),
    readFile(path.join(projectDir, "agent_output.log"), "utf8"),
  ]);
  return { projectId, projectDir, plan, prd, config, prompt, outputLog };
}

async function completeDescribeStep(projectName: string, description: string) {
  await expect(browser).toHaveTitle("LoopForge");
  await (await $('[data-testid="home-start-project-button"]')).click();
  await (await $('[data-testid="describe-page"]')).waitForDisplayed({ timeout: 30000 });
  await (await $('input[placeholder="e.g. auth-refactor"]')).setValue(projectName);
  await (await $('input[placeholder="/absolute/path/to/project"]')).setValue(
    readEnv("LOOPFORGE_E2E_WORKSPACE_DIR"),
  );
  await (await $('textarea[placeholder^="Describe what you want to build."]')).setValue(description);
  await (await $('[data-testid="describe-next-button"]')).click();
}

async function completePlanStep(projectId: string) {
  await (await $('[data-testid="plan-stream-panel"]')).waitForDisplayed({ timeout: 30000 });
  await browser.waitUntil(async () => {
    const nextButton = await waitForButton("Next");
    const planContent = await readFile(path.join(projectsDir(), projectId, "plan.md"), "utf8");
    return (await nextButton.isEnabled()) && planContent.trim().length > 0;
  }, { timeout: 30000, interval: 250, timeoutMsg: "plan never completed" });
  await (await waitForButton("Next")).click();
}

async function completeAtomizeStep(projectId: string) {
  await browser.waitUntil(async () => {
    const stories = await readJson<{ stories: Array<unknown> }>(
      path.join(projectsDir(), projectId, "prd.json"),
    );
    return stories.stories.length > 0;
  }, { timeout: 30000, interval: 250, timeoutMsg: "atomizer never produced stories" });
  await (await waitForButton("Proceed to validation")).click();
}

async function completeConfigureStep() {
  await (await $("h2=Execution config")).waitForDisplayed({ timeout: 30000 });
  await (await waitForButton("Next")).click();
}

async function completeLaunchStep(projectName: string, projectId: string) {
  await (await $("h2=Launch review")).waitForDisplayed({ timeout: 30000 });
  await (await waitForButton("Launch loop")).click();
  await browser.waitUntil(async () => {
    const heading = await $("h2*=SESSION MONITOR:");
    return (await heading.isDisplayed()) && (await heading.getText()).includes(projectName);
  }, { timeout: 30000, interval: 250, timeoutMsg: "monitor view never opened" });
  await browser.waitUntil(async () => {
    const prd = await readJson<{ stories: Array<{ passes: boolean }> }>(
      path.join(projectsDir(), projectId, "prd.json"),
    );
    return prd.stories.length > 0 && prd.stories.every((story) => story.passes);
  }, { timeout: 30000, interval: 250, timeoutMsg: "loop never marked stories as passed" });
}

export async function runDesktopHappyPath(projectName: string, description: string) {
  await completeDescribeStep(projectName, description);
  const projectId = await waitForSingleProjectId();
  await completePlanStep(projectId);
  await completeAtomizeStep(projectId);
  await completeConfigureStep();
  await completeLaunchStep(projectName, projectId);
  return readArtifacts(projectId);
}
