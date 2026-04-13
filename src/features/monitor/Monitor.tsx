import { useCallback, useEffect, useMemo, useState } from "react";
import { Navigate, useParams } from "react-router";
import { EphemeralOverlay } from "../../components/EphemeralOverlay";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../../components/ui/tabs";
import { useProjectSnapshot } from "../../hooks/useProjectSnapshot";
import { useDisplayVocabularyStore } from "../../stores/displayVocabularyStore";
import { useProjectStore } from "../../stores/projectStore";
import { useWizardDefaultsStore } from "../../stores/wizardDefaultsStore";
import { ActivityTab } from "./ActivityTab";
import { AskTab } from "./AskTab";
import { ExecutionProfilePanel } from "./components/ExecutionProfilePanel";
import { MonitorToolbar } from "./components/MonitorToolbar";
import { RetryButton } from "./components/RetryButton";
import { useMonitorActions } from "./hooks/useMonitorActions";
import { OutputTab } from "./OutputTab";
import { ProgressTab } from "./ProgressTab";

type MonitorTab = "progress" | "activity" | "output" | "ask" | "config";
const FALLBACK_TAB_ITEMS: Array<{ id: MonitorTab; label: string; disableForInactive?: boolean }> = [
  { id: "progress", label: "Progress" },
  { id: "activity", label: "Activity" },
  { id: "output", label: "Output" },
  { id: "ask", label: "Ask", disableForInactive: true },
  { id: "config", label: "Config" },
];
const FALLBACK_INACTIVE_STATUSES = ["archived", "failed"];

export function Monitor() {
  const { id } = useParams<{ id: string }>();
  const [activeTab, setActiveTab] = useState<MonitorTab>("progress");
  const [ephemeralOpen, setEphemeralOpen] = useState(false);
  const { snapshot, loading, error, refresh } = useProjectSnapshot(id);
  const fetchProjects = useProjectStore((state) => state.fetchProjects);
  const tabItems = useWizardDefaultsStore(
    (state) => state.defaults?.monitorTabs ?? FALLBACK_TAB_ITEMS,
  );
  const inactiveStatuses = useDisplayVocabularyStore(
    (state) => state.vocabulary?.inactiveStatuses ?? FALLBACK_INACTIVE_STATUSES,
  );
  const inactiveStatusSet = useMemo(() => new Set(inactiveStatuses), [inactiveStatuses]);
  const toggleEphemeral = useCallback(() => {
    setEphemeralOpen((current) => !current);
  }, []);
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
  const progress = snapshot?.progressPercent ?? 0;
  const monitorProjectId = snapshot?.project.id ?? "";
  const { busyAction, runAction } = useMonitorActions({
    projectId: monitorProjectId,
    refresh,
    fetchProjects,
  });
  if (!id) {
    return <Navigate to="/" replace />;
  }
  if (loading && !snapshot) {
    return (
      <div className="flex h-full items-center justify-center">
        <p className="font-sans text-sm text-text-dim">Loading project snapshot...</p>
      </div>
    );
  }

  if (!snapshot) {
    const notFound = Boolean(error?.includes("Project not found"));
    const errorMessage = notFound
      ? `Project not found: ${id}`
      : `Unable to load project snapshot: ${error ?? "unknown error"}`;
    return (
      <div className="flex h-full items-center justify-center">
        <div className="flex flex-col items-center gap-2">
          <p className="font-sans text-sm text-text-muted">{errorMessage}</p>
          <RetryButton
            variant="inline"
            onRetry={() => {
              void refresh();
            }}
          />
        </div>
      </div>
    );
  }
  if (snapshot.status === "draft") {
    const wizardStep = snapshot.project.wizardStep ?? "describe";
    const wizardPath =
      wizardStep === "describe" ? "/new/describe" : `/new/${wizardStep}/${snapshot.project.id}`;
    return <Navigate to={wizardPath} replace />;
  }
  const currentSnapshot = snapshot;
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
        onPauseResume={() => {
          void runAction(isPaused ? "resume" : "pause");
        }}
        onStop={() => {
          void runAction("stop");
        }}
      />
      {error && (
        <div className="flex items-center justify-between gap-3 border-paused/30 border-b bg-paused/10 px-6 py-2">
          <p className="truncate font-sans text-paused text-xs">
            Snapshot refresh warning: {error}
          </p>
          <RetryButton
            variant="banner"
            onRetry={() => {
              void refresh();
            }}
          />
        </div>
      )}
      <Tabs
        className="min-h-0 flex-1 gap-0"
        value={activeTab}
        onValueChange={(value) => {
          if (tabItems.some((tab) => tab.id === value)) {
            setActiveTab(value as MonitorTab);
          }
        }}
      >
        <div className="border-border border-b bg-surface/30 px-6">
          <TabsList className="w-full justify-start gap-1 rounded-none border-0 bg-transparent p-0">
            {tabItems.map((tab) => {
              const isDisabled =
                tab.disableForInactive && inactiveStatusSet.has(currentSnapshot.status);
              return (
                <TabsTrigger
                  key={tab.id}
                  value={tab.id}
                  disabled={isDisabled}
                  className="rounded-none border-0 border-transparent border-b-2 bg-transparent px-4 py-2.5 font-normal font-sans text-xs shadow-none disabled:cursor-not-allowed disabled:opacity-30 data-[state=active]:border-primary data-[state=active]:bg-transparent data-[state=active]:text-primary data-[state=active]:shadow-none"
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
            disabled={inactiveStatusSet.has(currentSnapshot.status)}
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
              <p className="font-sans text-text-dim text-xs">No configuration available.</p>
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
