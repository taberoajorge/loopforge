import { spawn } from "node:child_process";

const devUrl = "http://127.0.0.1:1420";

async function hasServerOnDevUrl() {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), 1500);
  try {
    const response = await fetch(devUrl, { signal: controller.signal });
    return response.status < 500;
  } catch {
    return false;
  } finally {
    clearTimeout(timer);
  }
}

function runVite() {
  const executable = process.platform === "win32" ? "bun.exe" : "bun";
  const child = spawn(executable, ["x", "vite"], {
    stdio: "inherit",
    shell: process.platform === "win32",
  });
  const stop = (signal) => {
    child.kill(signal);
  };
  process.on("SIGINT", () => stop("SIGINT"));
  process.on("SIGTERM", () => stop("SIGTERM"));
  child.on("exit", (code) => {
    process.exit(code ?? 1);
  });
}

const isRunning = await hasServerOnDevUrl();

if (isRunning) {
  process.stdout.write("Using existing Vite dev server on port 1420\n");
  process.exit(0);
}

runVite();
