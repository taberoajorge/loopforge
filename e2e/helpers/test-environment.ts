import { chmod, mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

const FIXTURE_BINARIES = ["claude", "codex", "cursor", "cursor-agent", "gemini", "opencode"];
const FIXTURE_AGENT_SCRIPT = `#!/bin/sh
all="$*"
fixture_set="\${LOOPFORGE_TEST_FIXTURE_SET:-happy-path}"
if printf '%s' "$all" | grep -q "Condense the following implementation plan"; then
  printf '%s\\n' 'Create the project, atomize the plan, execute the story, and archive the result.'
elif printf '%s' "$all" | grep -q "Split the following implementation plan"; then
  if [ "$fixture_set" = "atomize-error" ]; then
    printf '%s\\n' '{broken'
  else
    printf '%s\\n' '[{"title":"Lifecycle","content":"Create the project, atomize the plan, execute the story, and archive the result."}]'
  fi
elif printf '%s' "$all" | grep -q "decomposing a plan section into atomic user stories"; then
  if [ "$fixture_set" = "no-stories" ]; then
    printf '%s\\n' '[]'
  else
    printf '%s\\n' '[{"title":"Exercise desktop happy path","description":"Cover the desktop happy path flow.","acceptanceCriteria":["The desktop flow completes","Artifacts are persisted"],"scope":{"filesToModify":["e2e/wdio.conf.ts"],"filesToCreate":["e2e/specs/happy-path.e2e.ts"],"filesToAvoid":[]},"verification":{"commands":["bun run e2e -- --spec e2e/specs/happy-path.e2e.ts"],"assertions":[]},"commitMessage":"test(frontend): add desktop happy path spec","priority":"critical","estimatedComplexity":"medium","estimatedMinutes":45,"dependsOn":[]}]'
  fi
elif printf '%s' "$all" | grep -q "technical lead finalizing"; then
  if [ "$fixture_set" = "no-stories" ]; then
    printf '%s\\n' '{"projectName":"LoopForge Desktop Empty Result","feature":"Desktop atomize empty result","workingDirectory":"","generatedAt":"2026-04-10T00:00:00.000Z","stories":[],"totalEstimatedMinutes":0}'
  else
    printf '%s\\n' '{"projectName":"LoopForge Desktop Happy Path","feature":"Desktop happy path","workingDirectory":"","generatedAt":"2026-04-10T00:00:00.000Z","stories":[{"id":"S-001","title":"Exercise desktop happy path","description":"Cover the desktop happy path flow.","acceptanceCriteria":["The desktop flow completes","Artifacts are persisted"],"scope":{"filesToModify":["e2e/wdio.conf.ts"],"filesToCreate":["e2e/specs/happy-path.e2e.ts"],"filesToAvoid":[]},"verification":{"commands":["bun run e2e -- --spec e2e/specs/happy-path.e2e.ts"],"assertions":[]},"commitMessage":"test(frontend): add desktop happy path spec","priority":"critical","estimatedComplexity":"medium","estimatedMinutes":45,"dependsOn":[],"passes":false,"blocked":false,"attempts":0,"notes":null}]}'
  fi
else
  printf '%s\\n' 'fixture agent completed'
fi
`;

export type DesktopTestEnvironment = {
  rootDir: string;
  homeDir: string;
  binDir: string;
  workspaceDir: string;
  dialogDir: string;
  loopforgeDataDir: string;
  xdgConfigHome: string;
  xdgDataHome: string;
  env: NodeJS.ProcessEnv;
};

async function installFixtureAgents(binDir: string) {
  await Promise.all(
    FIXTURE_BINARIES.map(async (binaryName) => {
      const targetPath = path.join(binDir, binaryName);
      await writeFile(targetPath, FIXTURE_AGENT_SCRIPT);
      await chmod(targetPath, 0o755);
    }),
  );
}

export async function createDesktopTestEnvironment(): Promise<DesktopTestEnvironment> {
  const rootDir = await mkdtemp(path.join(os.tmpdir(), "loopforge-e2e-"));
  const fixtureSet = process.env.LOOPFORGE_E2E_FIXTURE_SET ?? "happy-path";
  const rustupHome = process.env.RUSTUP_HOME ?? path.join(os.homedir(), ".rustup");
  const cargoHome = process.env.CARGO_HOME ?? path.join(os.homedir(), ".cargo");
  const tauriDriverPath =
    process.env.TAURI_DRIVER_PATH ??
    path.join(
      cargoHome,
      "bin",
      process.platform === "win32" ? "tauri-driver.exe" : "tauri-driver",
    );
  const homeDir = path.join(rootDir, "home");
  const binDir = path.join(rootDir, "bin");
  const workspaceDir = path.join(rootDir, "workspace");
  const xdgConfigHome = path.join(rootDir, "xdg-config");
  const xdgDataHome = path.join(rootDir, "xdg-data");
  const loopforgeDataDir = path.join(rootDir, "loopforge-test-data");
  const dialogDir = path.join(rootDir, "loopforge-test-dialogs");
  await Promise.all([
    mkdir(homeDir, { recursive: true }),
    mkdir(binDir, { recursive: true }),
    mkdir(workspaceDir, { recursive: true }),
    mkdir(xdgConfigHome, { recursive: true }),
    mkdir(xdgDataHome, { recursive: true }),
    mkdir(loopforgeDataDir, { recursive: true }),
    mkdir(dialogDir, { recursive: true }),
  ]);
  await installFixtureAgents(binDir);
  const fixturePath = [binDir, process.env.PATH].filter(Boolean).join(path.delimiter);
  return {
    rootDir,
    homeDir,
    binDir,
    workspaceDir,
    dialogDir,
    loopforgeDataDir,
    xdgConfigHome,
    xdgDataHome,
    env: {
      ...process.env,
      HOME: homeDir,
      PATH: fixturePath,
      RUSTUP_HOME: rustupHome,
      CARGO_HOME: cargoHome,
      TAURI_DRIVER_PATH: tauriDriverPath,
      LOOPFORGE_TEST_MODE: "1",
      LOOPFORGE_TEST_FIXTURE_SET: fixtureSet,
      LOOPFORGE_TEST_DATA_DIR: loopforgeDataDir,
      LOOPFORGE_TEST_DIALOG_DIR: dialogDir,
      XDG_CONFIG_HOME: xdgConfigHome,
      XDG_DATA_HOME: xdgDataHome,
    },
  };
}

export async function destroyDesktopTestEnvironment(
  environment: DesktopTestEnvironment | undefined,
): Promise<void> {
  if (!environment) {
    return;
  }
  await rm(environment.rootDir, { recursive: true, force: true });
}
