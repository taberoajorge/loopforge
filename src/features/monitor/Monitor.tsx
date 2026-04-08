import { useCallback, useEffect, useMemo, useState } from "react";
import { Navigate, useParams } from "react-router";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../../components/ui/tabs";
import { EphemeralOverlay } from "../../components/EphemeralOverlay";
import { pauseProject, resumeProject, stopLoop } from "../../lib/tauri";
import { useProjectSnapshot } from "../../hooks/useProjectSnapshot";
import { useProjectStore } from "../../stores/projectStore";
import { ActivityTab } from "./ActivityTab";
import { AskTab } from "./AskTab";
import { ExecutionProfilePanel } from "./components/ExecutionProfilePanel";
import { MonitorToolbar } from "./components/MonitorToolbar";
import { OutputTab } from "./OutputTab";
import { ProgressTab } from "./ProgressTab";
type MonitorTab = "progress" | "activity" | "output" | "ask" | "config";
const TAB_ITEMS: Array<{ id: MonitorTab; label: string; disableForInactive?: boolean }> = [
  { id: "progress", label: "Progress" },
  { id: "activity", label: "Activity" },
  { id: "output", label: "Output" },
  { id: "ask", label: "Ask", disableForInactive: true },
  { id: "config", label: "Config" },
];

const INACTIVE_STATUSES = new Set(["archived", "failed"]);

function isMonitorTab(value: string): value is MonitorTab {
  return value === "progress" || value === "activity" || value === "output" || value === "ask" || value === "config";
}
export function Monitor() {
  const { id } = useParams<{ id: string }>();
  const [activeTab, setActiveTab] = useState<MonitorTab>("progress");
  const [ephemeralOpen, setEphemeralOpen] = useState(false);
  const [busyAction, setBusyAction] = useState<"pause" | "resume" | "stop" | null>(null);
  const { snapshot, loading, error, refresh } = useProjectSnapshot(id);
  const fetchProjects = useProjectStore((state) => state.fetchProjects);
  const toggleEphemeral = useCallback(() => { setEphemeralOpen((current) => !current); }, []);
  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.ctrlKey && event.shiftKey && event.code === "Space") {
        event.preventDefault();
        toggleEphemeral();
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [toggleEphemeral]);
  const progress = useMemo(() => {
    if (!snapshot || snapshot.progress.storiesTotal === 0) {
      return 0;
    }
    return Math.round(
      (snapshot.progress.storiesDone / snapshot.progress.storiesTotal) * 100,
    );
  }, [snapshot]);
  if (!id) {
    return <Navigate to="/" replace />;
  }
  if (loading && !snapshot) {
    return (
      <div className="flex items-center justify-center h-full">
        <p className="text-text-dim text-sm font-sans">Loading project snapshot...</p>
      </div>
    );
  }

  if (!snapshot) {
    const notFound = Boolean(error?.includes("Project not found"));
    const errorMessage = notFound
      ? `Project not found: ${id}`
      : `Unable to load project snapshot: ${error ?? "unknown error"}`;
    return (
      <div className="flex items-center justify-center h-full">
        <div className="flex flex-col items-center gap-2">
          <p className="text-text-muted text-sm font-sans">{errorMessage}</p>
          <button
            onClick={() => {
              void refresh();
            }}
            className="px-3 py-1.5 rounded bg-elevated text-text-muted text-xs font-sans hover:text-text transition-colors"
          >
            Retry
          </button>
        </div>
      </div>
    );
  }
  if (snapshot.status === "draft") {
    const wizardStep = snapshot.project.wizardStep ?? "describe";
    const wizardPath = wizardStep === "describe"
      ? "/new/describe"
      : `/new/${wizardStep}/${snapshot.project.id}`;
    return <Navigate to={wizardPath} replace />;
  }
  const currentSnapshot = snapshot;
  async function runAction(action: "pause" | "resume" | "stop") {
    setBusyAction(action);
    try {
      if (action === "pause") {
        await pauseProject(currentSnapshot.project.id);
      } else if (action === "resume") {
        await resumeProject(currentSnapshot.project.id);
      } else {
        await stopLoop(currentSnapshot.project.id);
      }
      await refresh();
      await fetchProjects();
    } finally {
      setBusyAction(null);
    }
  }
  const isRunning = currentSnapshot.status === "running";
  const isPaused = currentSnapshot.status === "paused";
  return (
    <div className="flex h-full min-h-0 flex-col">
      <MonitorToolbar
        title={currentSnapshot.project.name}
        status={currentSnapshot.status}
        storiesDone={currentSnapshot.progress.storiesDone}
        storiesTotal={currentSnapshot.progress.storiesTotal}
        storyHint={currentSnapshot.progress.currentStory}
        progress={progress}
        isRunning={isRunning}
        isPaused={isPaused}
        busyAction={busyAction}
        onPauseResume={() => runAction(isPaused ? "resume" : "pause")}
        onStop={() => runAction("stop")}
      />
      {error && (
        <div className="bg-paused/10 border-b border-paused/30 px-6 py-2 flex items-center justify-between gap-3">
          <p className="text-paused text-xs font-sans truncate">
            Snapshot refresh warning: {error}
          </p>
          <button
            onClick={() => {
              void refresh();
            }}
            className="px-3 py-1 rounded bg-surface border border-border text-text-muted text-xs font-sans hover:text-text transition-colors"
          >
            Retry
          </button>
        </div>
      )}
      <Tabs
        className="flex-1 min-h-0 gap-0"
        value={activeTab}
        onValueChange={(value) => {
          if (isMonitorTab(value)) {
            setActiveTab(value);
          }
        }}
      >
        <div className="border-b border-border bg-surface/30 px-6">
          <TabsList className="w-full justify-start border-0 bg-transparent p-0 rounded-none gap-1">
            {TAB_ITEMS.map((tab) => {
              const isDisabled = tab.disableForInactive && INACTIVE_STATUSES.has(currentSnapshot.status);
              return (
                <TabsTrigger
                  key={tab.id}
                  value={tab.id}
                  disabled={isDisabled}
                  className="rounded-none border-0 border-b-2 border-transparent bg-transparent px-4 py-2.5 text-xs font-sans font-normal shadow-none data-[state=active]:border-primary data-[state=active]:bg-transparent data-[state=active]:text-primary data-[state=active]:shadow-none disabled:opacity-30 disabled:cursor-not-allowed"
                >
                  {tab.label}
                </TabsTrigger>
              );
            })}
          </TabsList>
        </div>
        <TabsContent value="progress" className="h-full overflow-hidden">
          <ProgressTab projectId={currentSnapshot.project.id} />
        </TabsContent>
        <TabsContent value="activity" className="h-full overflow-hidden">
          <ActivityTab projectId={currentSnapshot.project.id} />
        </TabsContent>
        <TabsContent value="output" className="h-full overflow-hidden">
          <OutputTab projectId={currentSnapshot.project.id} />
        </TabsContent>
        <TabsContent value="ask" className="h-full overflow-hidden">
          <AskTab
            projectId={currentSnapshot.project.id}
            disabled={INACTIVE_STATUSES.has(currentSnapshot.status)}
          />
        </TabsContent>
        <TabsContent value="config" className="h-full overflow-auto">
          {currentSnapshot.config ? (
            <ExecutionProfilePanel
              projectId={currentSnapshot.project.id}
              config={currentSnapshot.config}
              isPaused={isPaused}
              onSaved={async () => {
                await refresh();
                await fetchProjects();
              }}
            />
          ) : (
            <div className="px-6 py-4">
              <p className="text-text-dim text-xs font-sans">No configuration available.</p>
            </div>
          )}
        </TabsContent>
      </Tabs>
      <EphemeralOverlay
        projectId={currentSnapshot.project.id}
        isOpen={ephemeralOpen}
        onClose={() => setEphemeralOpen(false)}
      />
    </div>
  );
}
