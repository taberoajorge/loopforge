import { spawn, spawnSync, type ChildProcess } from "node:child_process";
import { access } from "node:fs/promises";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  createDesktopTestEnvironment,
  destroyDesktopTestEnvironment,
  type DesktopTestEnvironment,
} from "./helpers/test-environment.ts";

let tauriDriver: ChildProcess | undefined;
let testEnvironment: DesktopTestEnvironment | undefined;

const projectRoot = fileURLToPath(new URL("..", import.meta.url));
const driverPort = Number(process.env.LOOPFORGE_E2E_DRIVER_PORT ?? "4444");
const appBinaryName = process.platform === "win32" ? "loopforge.exe" : "loopforge";
const applicationPath =
  process.env.LOOPFORGE_E2E_APP_PATH ??
  path.join(projectRoot, "target", "debug", appBinaryName);
const tauriDriverPath =
  process.env.TAURI_DRIVER_PATH ??
  path.join(
    process.env.CARGO_HOME ?? path.join(os.homedir(), ".cargo"),
    "bin",
    process.platform === "win32" ? "tauri-driver.exe" : "tauri-driver",
  );

function buildDesktopApp() {
  if (process.env.LOOPFORGE_E2E_SKIP_BUILD === "1") {
    return;
  }
  const build = spawnSync(
    "bun",
    ["run", "tauri", "build", "--debug", "--no-bundle"],
    { cwd: projectRoot, stdio: "inherit" },
  );
  if (build.status !== 0) {
    throw new Error(`Tauri build failed with exit code ${build.status ?? "unknown"}`);
  }
}

async function assertPathExists(filePath: string, label: string) {
  try {
    await access(filePath);
  } catch {
    throw new Error(`${label} not found at ${filePath}`);
  }
}

async function waitForDriverReady(port: number) {
  const deadline = Date.now() + 10000;
  while (Date.now() < deadline) {
    const connected = await new Promise<boolean>((resolve) => {
      const socket = net.createConnection({ host: "127.0.0.1", port });
      socket.once("connect", () => {
        socket.end();
        resolve(true);
      });
      socket.once("error", () => resolve(false));
    });
    if (connected) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error(`tauri-driver did not start on port ${port}`);
}

function killTauriDriver() {
  if (!tauriDriver) {
    return;
  }
  tauriDriver.kill();
  tauriDriver = undefined;
}

export const config = {
  runner: "local",
  specs: ["./e2e/specs/**/*.e2e.ts"],
  maxInstances: 1,
  hostname: "127.0.0.1",
  port: driverPort,
  path: "/",
  logLevel: "warn",
  waitforTimeout: 20000,
  connectionRetryTimeout: 120000,
  reporters: ["spec"],
  framework: "mocha",
  mochaOpts: {
    ui: "bdd",
    timeout: 60000,
  },
  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": {
        application: applicationPath,
      },
    },
  ],
  async onPrepare() {
    testEnvironment = await createDesktopTestEnvironment();
    buildDesktopApp();
    await assertPathExists(tauriDriverPath, "tauri-driver");
    await assertPathExists(applicationPath, "desktop app binary");
  },
  async beforeSession() {
    tauriDriver = spawn(tauriDriverPath, [], {
      env: testEnvironment?.env ?? process.env,
      stdio: "inherit",
    });
    await waitForDriverReady(driverPort);
  },
  afterSession() {
    killTauriDriver();
  },
  async onComplete() {
    killTauriDriver();
    await destroyDesktopTestEnvironment(testEnvironment);
    testEnvironment = undefined;
  },
};
