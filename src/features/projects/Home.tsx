import { useCallback, useEffect, useState } from "react";
import { useNavigate } from "react-router";
import { EmptyState } from "../../components/EmptyState";
import { SectionHeader } from "../../components/SectionHeader";
import { reportError } from "../../lib/reportError";
import { checkSystemReadiness, discardDraft, type SystemReadiness } from "../../lib/tauri";
import { useProjectStore } from "../../stores/projectStore";
import { ActiveLoopCard } from "./components/ActiveLoopCard";
import { CompletedLoopCard } from "./components/CompletedLoopCard";
import { DraftCard } from "./components/DraftCard";
import { SystemReadinessPanel } from "./components/SystemReadinessPanel";
import { SystemStatusCard } from "./components/SystemStatusCard";

const READINESS_DISMISSED_KEY = "loopforge.readiness.dismissed";

function useTypewriter(text: string, speed: number = 80): string {
  const [displayed, setDisplayed] = useState("");
  useEffect(() => {
    setDisplayed("");
    let index = 0;
    const interval = setInterval(() => {
      if (index < text.length) {
        setDisplayed(text.slice(0, index + 1));
        index++;
      } else {
        clearInterval(interval);
      }
    }, speed);
    return () => clearInterval(interval);
  }, [text, speed]);
  return displayed;
}

export function Home() {
  const navigate = useNavigate();
  const projects = useProjectStore((state) => state.projects);
  const grouped = useProjectStore((state) => state.grouped);
  const loading = useProjectStore((state) => state.loading);
  const fetchProjects = useProjectStore((state) => state.fetchProjects);
  const heading = useTypewriter("INITIALIZE SEQUENCE", 60);
  const [systemReadiness, setSystemReadiness] = useState<SystemReadiness | null>(null);
  const [readinessVisible, setReadinessVisible] = useState(false);
  const [readinessLoading, setReadinessLoading] = useState(false);

  const refreshSystemReadiness = useCallback(async () => {
    setReadinessLoading(true);
    try {
      const readiness = await checkSystemReadiness();
      const hasBlockingIssue =
        !readiness.gitAvailable ||
        !readiness.shellAvailable ||
        readiness.agents.every((agent) => !agent.available);
      setSystemReadiness(readiness);
      setReadinessVisible(hasBlockingIssue);
    } catch (caughtError) {
      reportError("Home.checkSystemReadiness", caughtError);
      setReadinessVisible(false);
    } finally {
      setReadinessLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchProjects();
  }, [fetchProjects]);

  useEffect(() => {
    if (window.localStorage.getItem(READINESS_DISMISSED_KEY) === "1") {
      return;
    }
    refreshSystemReadiness();
  }, [refreshSystemReadiness]);

  const {
    active: activeProjects,
    drafts: draftProjects,
    finished: finishedProjects,
    archived: archivedProjects,
  } = grouped;

  async function handleDiscardDraft(projectId: string) {
    await discardDraft(projectId).catch((caughtError: unknown) => {
      reportError("Home.discardDraft", caughtError);
    });
    fetchProjects();
  }

  function dismissReadiness() {
    window.localStorage.setItem(READINESS_DISMISSED_KEY, "1");
    setReadinessVisible(false);
  }

  return (
    <div className="flex h-full flex-col overflow-hidden" data-testid="home-page">
      <div className="grid shrink-0 items-start gap-6 p-6 pb-2 xl:grid-cols-[minmax(0,1fr)_17.5rem]">
        <div className="px-1 py-1">
          <h1 className="mb-1 font-bold font-mono text-3xl text-glow-primary text-primary tracking-widest">
            {heading}
            <span className="animate-pulse text-primary">_</span>
          </h1>
          <p className="mb-4 font-sans text-sm text-text-muted">
            Autonomous AI loop orchestrator. Plan. Atomize. Execute. Monitor.
          </p>
          <button
            type="button"
            onClick={() => navigate("/new/describe")}
            data-testid="home-start-project-button"
            className="rounded-md bg-primary px-8 py-3 font-bold font-sans text-sm text-void uppercase tracking-wider shadow-glow-primary transition-all hover:brightness-110"
          >
            Start new project
          </button>
        </div>
        <SystemStatusCard activeCount={activeProjects.length} totalCount={projects.length} />
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto px-6 pb-2">
        <div className="space-y-6">
          {readinessVisible && systemReadiness ? (
            <SystemReadinessPanel
              readiness={systemReadiness}
              refreshing={readinessLoading}
              onRefresh={refreshSystemReadiness}
              onDismiss={dismissReadiness}
            />
          ) : null}
          {draftProjects.length > 0 ? (
            <section>
              <SectionHeader title="DRAFTS" compact className="mb-2" />
              <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
                {draftProjects.map((project) => (
                  <DraftCard key={project.id} project={project} onDiscard={handleDiscardDraft} />
                ))}
              </div>
            </section>
          ) : null}
          <section>
            <SectionHeader title="ACTIVE LOOPS" compact className="mb-2" />
            {activeProjects.length > 0 ? (
              <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
                {activeProjects.map((project) => (
                  <ActiveLoopCard key={project.id} project={project} />
                ))}
              </div>
            ) : (
              <EmptyState
                title="No active loops"
                description="Start a new project to launch your first execution loop."
                compact
                padding="tight"
              />
            )}
          </section>
          {finishedProjects.length > 0 ? (
            <section>
              <SectionHeader title="FINISHED LOOPS" compact className="mb-2" />
              <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
                {finishedProjects.map((project) => (
                  <CompletedLoopCard key={project.id} project={project} />
                ))}
              </div>
            </section>
          ) : null}
          {archivedProjects.length > 0 ? (
            <section>
              <SectionHeader title="ARCHIVED LOOPS" compact className="mb-2" />
              <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
                {archivedProjects.map((project) => (
                  <CompletedLoopCard key={project.id} project={project} />
                ))}
              </div>
            </section>
          ) : null}
          {!loading && projects.length === 0 ? (
            <EmptyState
              title="No projects yet"
              description="Start your first loop from the launch control above."
              primaryAction={{
                label: "Start new project",
                onClick: () => navigate("/new/describe"),
              }}
            />
          ) : null}
        </div>
      </div>
      <div className="flex shrink-0 items-center justify-between border-border border-t bg-surface/50 px-6 py-2 font-mono text-text-dim text-xs">
        <span>LOOPFORGE v0.1.0</span>
        <span>
          {activeProjects.length} ACTIVE | {finishedProjects.length} FINISHED |{" "}
          {archivedProjects.length} ARCHIVED | {draftProjects.length} DRAFT
        </span>
      </div>
    </div>
  );
}
