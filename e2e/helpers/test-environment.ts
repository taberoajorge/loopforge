import { mkdtemp, mkdir, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

export type DesktopTestEnvironment = {
  rootDir: string;
  dialogDir: string;
  loopforgeDataDir: string;
  xdgConfigHome: string;
  xdgDataHome: string;
  env: NodeJS.ProcessEnv;
};

export async function createDesktopTestEnvironment(): Promise<DesktopTestEnvironment> {
  const rootDir = await mkdtemp(path.join(os.tmpdir(), "loopforge-e2e-"));
  const xdgConfigHome = path.join(rootDir, "xdg-config");
  const xdgDataHome = path.join(rootDir, "xdg-data");
  const loopforgeDataDir = path.join(rootDir, "loopforge-test-data");
  const dialogDir = path.join(rootDir, "loopforge-test-dialogs");
  await Promise.all([
    mkdir(xdgConfigHome, { recursive: true }),
    mkdir(xdgDataHome, { recursive: true }),
    mkdir(loopforgeDataDir, { recursive: true }),
    mkdir(dialogDir, { recursive: true }),
  ]);
  return {
    rootDir,
    dialogDir,
    loopforgeDataDir,
    xdgConfigHome,
    xdgDataHome,
    env: {
      ...process.env,
      LOOPFORGE_TEST_MODE: "1",
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
