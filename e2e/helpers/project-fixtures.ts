import assert from "node:assert/strict";
import { access, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";
import { browser } from "@wdio/globals";

type SeedProjectArgs = {
  id: string;
  name: string;
  description: string;
  wizardStep: "describe" | "plan" | "atomize" | "configure" | "launch";
  status?: "draft" | "ready" | "paused" | "blocked" | "failed" | "completed" | "archived";
  draft: Record<string, unknown>;
  plan?: string;
  prd?: string;
  config?: string;
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

function dbPath(): string {
  return path.join(appDataDir(), "loopforge.db");
}

export function projectsDir(): string {
  return path.join(appDataDir(), "projects");
}

export function projectDir(projectId: string): string {
  return path.join(projectsDir(), projectId);
}

async function pathExists(targetPath: string): Promise<boolean> {
  try {
    await access(targetPath);
    return true;
  } catch {
    return false;
  }
}

async function waitForDatabase() {
  await browser.waitUntil(
    async () => pathExists(dbPath()),
    { timeout: 30000, interval: 250, timeoutMsg: "expected loopforge.db to exist" },
  );
}

function openDatabase(): DatabaseSync {
  return new DatabaseSync(dbPath());
}

function now(): string {
  return "2026-04-10T12:00:00.000Z";
}

async function writeOptionalFile(filePath: string, content: string | undefined) {
  if (content === undefined) {
    await rm(filePath, { force: true });
    return;
  }
  await writeFile(filePath, content);
}

export async function seedProjectFixture(args: SeedProjectArgs) {
  await waitForDatabase();
  const workspaceDir = readEnv("LOOPFORGE_E2E_WORKSPACE_DIR");
  const targetDir = projectDir(args.id);
  const status = args.status ?? "draft";
  await mkdir(targetDir, { recursive: true });
  await Promise.all([
    writeFile(path.join(targetDir, "draft.json"), JSON.stringify(args.draft, null, 2)),
    writeOptionalFile(path.join(targetDir, "plan.md"), args.plan),
    writeOptionalFile(path.join(targetDir, "prd.json"), args.prd),
    writeOptionalFile(path.join(targetDir, "config.json"), args.config),
  ]);

  const database = openDatabase();
  try {
    database.exec("PRAGMA journal_mode=WAL;");
    database.prepare(
      `INSERT OR REPLACE INTO projects (
        id, name, description, status, working_directory, created_at, updated_at, wizard_step, wizard_state_json
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, NULL)`,
    ).run(args.id, args.name, args.description, status, workspaceDir, now(), now(), args.wizardStep);
  } finally {
    database.close();
  }
}

export async function readProjectPlan(projectId: string): Promise<string> {
  return readFile(path.join(projectDir(projectId), "plan.md"), "utf8");
}

export async function projectRecordExists(projectId: string): Promise<boolean> {
  await waitForDatabase();
  const database = openDatabase();
  try {
    const row = database.prepare(
      "SELECT COUNT(*) AS total FROM projects WHERE id = ?",
    ).get(projectId) as { total: number };
    return row.total > 0;
  } finally {
    database.close();
  }
}

export async function projectArtifactExists(projectId: string): Promise<boolean> {
  return pathExists(projectDir(projectId));
}
