import { useEffect, useState } from "react";
import { useNavigate } from "react-router";
import { useProjectStore } from "../../stores/projectStore";
import { discardDraft } from "../../lib/tauri";
import { EmptyState } from "../../components/EmptyState";
import { SectionHeader } from "../../components/SectionHeader";
import { ActiveLoopCard } from "./components/ActiveLoopCard";
import { CompletedLoopCard } from "./components/CompletedLoopCard";
import { DraftCard } from "./components/DraftCard";
import { SystemStatusCard } from "./components/SystemStatusCard";
import { ACTIVE_PROJECT_STATUSES, FINISHED_PROJECT_STATUSES } from "../../lib/project-status";

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
  const loading = useProjectStore((state) => state.loading);
  const fetchProjects = useProjectStore((state) => state.fetchProjects);
  const heading = useTypewriter("INITIALIZE SEQUENCE", 60);

  useEffect(() => {
    fetchProjects();
  }, [fetchProjects]);

  const draftProjects = projects.filter((proj) => proj.status === "draft");
  const activeProjects = projects.filter((proj) => ACTIVE_PROJECT_STATUSES.includes(proj.status));
  const finishedProjects = projects.filter((proj) => FINISHED_PROJECT_STATUSES.includes(proj.status));
  const archivedProjects = projects.filter((proj) => proj.status === "archived");

  async function handleDiscardDraft(projectId: string) {
    await discardDraft(projectId).catch(() => {});
    fetchProjects();
  }

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <div className="shrink-0 grid items-start gap-6 p-6 pb-2 xl:grid-cols-[minmax(0,1fr)_17.5rem]">
        <div className="px-1 py-1">
          <h1 className="text-3xl font-mono font-bold text-primary text-glow-primary tracking-widest mb-1">
            {heading}
            <span className="animate-pulse text-primary">_</span>
          </h1>
          <p className="text-text-muted text-sm font-sans mb-4">
            Autonomous AI loop orchestrator. Plan. Atomize. Execute. Monitor.
          </p>
          <button
            onClick={() => navigate("/new/describe")}
            className="px-8 py-3 rounded-md bg-primary text-void font-bold font-sans text-sm shadow-glow-primary hover:brightness-110 transition-all uppercase tracking-wider"
          >
            Start new project
          </button>
        </div>
        <SystemStatusCard activeCount={activeProjects.length} totalCount={projects.length} />
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto px-6 pb-2">
        <div className="space-y-6">
          {draftProjects.length > 0 ? (
            <section>
              <SectionHeader title="DRAFTS" compact className="mb-2" />
              <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
                {draftProjects.map((project) => <DraftCard key={project.id} project={project} onDiscard={handleDiscardDraft} />)}
              </div>
            </section>
          ) : null}
          <section>
            <SectionHeader title="ACTIVE LOOPS" compact className="mb-2" />
            {activeProjects.length > 0 ? (
              <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
                {activeProjects.map((project) => <ActiveLoopCard key={project.id} project={project} />)}
              </div>
            ) : (
              <EmptyState title="No active loops" description="Start a new project to launch your first execution loop." compact padding="tight" />
            )}
          </section>
          {finishedProjects.length > 0 ? (
            <section>
              <SectionHeader title="FINISHED LOOPS" compact className="mb-2" />
              <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
                {finishedProjects.map((project) => <CompletedLoopCard key={project.id} project={project} />)}
              </div>
            </section>
          ) : null}
          {archivedProjects.length > 0 ? (
            <section>
              <SectionHeader title="ARCHIVED LOOPS" compact className="mb-2" />
              <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
                {archivedProjects.map((project) => <CompletedLoopCard key={project.id} project={project} />)}
              </div>
            </section>
          ) : null}
          {!loading && projects.length === 0 ? (
            <EmptyState
              title="No projects yet"
              description="Start your first loop from the launch control above."
              primaryAction={{ label: "Start new project", onClick: () => navigate("/new/describe") }}
            />
          ) : null}
        </div>
      </div>
      <div className="shrink-0 border-t border-border bg-surface/50 px-6 py-2 flex items-center justify-between text-xs font-mono text-text-dim">
        <span>LOOPFORGE v0.1.0</span>
        <span>{activeProjects.length} ACTIVE | {finishedProjects.length} FINISHED | {archivedProjects.length} ARCHIVED | {draftProjects.length} DRAFT</span>
      </div>
    </div>
  );
}
