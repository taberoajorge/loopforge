import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { initializeTheme } from "./stores/themeStore";
import "./index.css";

initializeTheme();

function NativeShellBoot() {
  return (
    <main className="min-h-screen bg-void text-primary flex items-center justify-center">
      <div className="text-center space-y-2">
        <p className="text-text-muted text-xs font-mono uppercase tracking-widest">
          LoopForge Native Shell
        </p>
        <p className="text-primary text-sm font-sans">
          Native shell is the primary runtime.
        </p>
      </div>
    </main>
  );
}

function shouldLoadLegacyRouter(): boolean {
  const isLegacyMode = import.meta.env.MODE === "legacy-router";
  const hasLegacyQuery = new URLSearchParams(window.location.search).has("legacy-router");
  return isLegacyMode || hasLegacyQuery;
}

async function mountApplication() {
  const rootElement = document.getElementById("root");
  if (!rootElement) {
    return;
  }

  if (shouldLoadLegacyRouter()) {
    const { mountLegacyRouter } = await import("./router");
    mountLegacyRouter(rootElement);
    return;
  }

  createRoot(rootElement).render(
    <StrictMode>
      <NativeShellBoot />
    </StrictMode>,
  );
}

void mountApplication();
