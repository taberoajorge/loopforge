import { useEffect, useRef, useState, type DragEvent } from "react";
import { useNavigate, useParams } from "react-router";
import { discardDraft, onAtomizationProgress, runAtomizer, saveDraft, savePrd } from "../../lib/tauri";
import type { AtomizeProgress, Prd } from "../../lib/tauri";
import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from "../../components/ui/card";
import { Progress } from "../../components/ui/progress";
import { ScrollArea, ScrollContent, ScrollViewport } from "../../components/ui/scroll-area";
import { useWizardStore, type UserStory } from "../../stores/wizardStore";
import { AtomizeConfirmationDialog } from "./components/AtomizeConfirmationDialog";
import { AtomizeStoryList } from "./components/AtomizeStoryList";

type StageStatus = "pending" | "running" | "done" | "error";
type PipelineStage = { number: number; label: string; status: StageStatus };

const INITIAL_STAGES: PipelineStage[] = [
  { number: 1, label: "Summarize", status: "pending" },
  { number: 2, label: "Chunk", status: "pending" },
  { number: 3, label: "Atomize", status: "pending" },
  { number: 4, label: "Merge", status: "pending" },
];
const STAGE_BADGE: Record<StageStatus, "neutral" | "info" | "success" | "danger"> = {
  pending: "neutral",
  running: "info",
  done: "success",
  error: "danger",
};
const STAGE_LABEL: Record<StageStatus, string> = {
  pending: "Pending",
  running: "Running",
  done: "Done",
  error: "Error",
};
const atomizerPromiseByProject = new Map<string, Promise<Prd>>();

function makeBlankStory(existingCount: number): UserStory {
  const paddedId = String(existingCount + 1).padStart(3, "0");
  return {
    id: `S-${paddedId}`,
    title: "New story",
    description: "",
    acceptanceCriteria: [],
    scope: { filesToModify: [], filesToCreate: [], filesToAvoid: [] },
    verification: { commands: [], assertions: [] },
    priority: "medium",
    estimatedComplexity: "medium",
    estimatedMinutes: 30,
    dependsOn: [],
    passes: false,
    blocked: false,
    attempts: 0,
    notes: null,
  };
}

export function Atomize() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const stories = useWizardStore((state) => state.stories);
  const reorderStories = useWizardStore((state) => state.reorderStories);
  const addStory = useWizardStore((state) => state.addStory);
  const updateStory = useWizardStore((state) => state.updateStory);
  const removeStory = useWizardStore((state) => state.removeStory);
  const advanceStep = useWizardStore((state) => state.advanceStep);
  const dragIndexRef = useRef<number | null>(null);
  const startedRef = useRef(false);
  const [stages, setStages] = useState(INITIAL_STAGES);
  const [atomizeStarted, setAtomizeStarted] = useState(false);
  const [atomizeError, setAtomizeError] = useState<string | null>(null);
  const [stageMessage, setStageMessage] = useState("");
  const [discardOpen, setDiscardOpen] = useState(false);
  const [storyToRemove, setStoryToRemove] = useState<UserStory | null>(null);
  const [elapsedSeconds, setElapsedSeconds] = useState(0);
  const totalMinutes = stories.reduce((sum, story) => sum + story.estimatedMinutes, 0);
  const totalHours = (totalMinutes / 60).toFixed(1);

  useEffect(() => {
    if (!id || startedRef.current) return;
    const snap = useWizardStore.getState();
    if (!snap.projectData.name) return;
    if (snap.stories.length > 0 && snap.projectId === id) {
      startedRef.current = true;
      setAtomizeStarted(true);
      setStages(INITIAL_STAGES.map((stage) => ({ ...stage, status: "done" })));
      setStageMessage("Loaded existing stories.");
      return;
    }
    startedRef.current = true;
    setAtomizeStarted(true);
    setStageMessage("Summarizing plan...");
    setStages(INITIAL_STAGES.map((stage) => ({ ...stage, status: stage.number === 1 ? "running" : "pending" })));
    let disposed = false;
    let listenerReleased = false;
    const unlistenPromise = onAtomizationProgress((progress: AtomizeProgress) => {
      if (progress.projectId !== id) return;
      setStageMessage(progress.message);
      setStages((previous) => previous.map((stage) => stage.number === progress.stage ? { ...stage, status: "running" } : stage.number < progress.stage ? { ...stage, status: "done" } : stage));
    });
    const releaseListener = () => {
      if (listenerReleased) return;
      listenerReleased = true;
      unlistenPromise.then((unlisten) => unlisten());
    };
    const inFlightPromise = atomizerPromiseByProject.get(id) ?? runAtomizer({ projectId: id, projectName: snap.projectData.name, projectDir: snap.projectData.workingDirectory, agent: snap.projectData.planAgent, model: snap.projectData.planModel, effort: snap.projectData.planEffort });
    if (!atomizerPromiseByProject.has(id)) atomizerPromiseByProject.set(id, inFlightPromise);
    inFlightPromise.then((prd) => {
      if (disposed) return;
      useWizardStore.getState().setStories(prd.stories);
      setStages(INITIAL_STAGES.map((stage) => ({ ...stage, status: "done" })));
      setStageMessage(`Done. ${prd.stories.length} stories generated.`);
    }).catch((error: unknown) => {
      if (disposed) return;
      setAtomizeError(error instanceof Error ? error.message : String(error));
      setStages((previous) => previous.map((stage) => stage.status === "running" ? { ...stage, status: "error" } : stage));
    }).finally(() => {
      if (atomizerPromiseByProject.get(id) === inFlightPromise) atomizerPromiseByProject.delete(id);
      releaseListener();
    });
    return () => {
      disposed = true;
      releaseListener();
    };
  }, [id]);

  const pipelineProgress = stages.reduce((sum, stage) => sum + (stage.status === "done" ? 1 : stage.status === "running" ? 0.5 : 0), 0);
  const pipelinePercent = Math.round((pipelineProgress / stages.length) * 100);
  const isDone = stages.every((stage) => stage.status === "done");
  const isRunning = atomizeStarted && !isDone && !atomizeError;
  const formatElapsed = (secs: number) => secs < 60 ? `${secs}s` : `${Math.floor(secs / 60)}m ${secs % 60}s`;

  useEffect(() => {
    if (!isRunning) return;
    setElapsedSeconds(0);
    const timer = setInterval(() => setElapsedSeconds((prev) => prev + 1), 1000);
    return () => clearInterval(timer);
  }, [isRunning]);

  const handleDragStart = (index: number) => { dragIndexRef.current = index; };
  const handleDragOver = (event: DragEvent) => { event.preventDefault(); };
  const handleDrop = (toIndex: number) => {
    if (dragIndexRef.current === null || dragIndexRef.current === toIndex) return;
    reorderStories(dragIndexRef.current, toIndex);
    dragIndexRef.current = null;
  };

  async function persistStoriesToDisk() {
    const snap = useWizardStore.getState();
    if (!id || snap.stories.length === 0) return;
    const prd: Prd = { projectName: snap.projectData.name, generatedAt: new Date().toISOString(), totalEstimatedMinutes: totalMinutes, stories: snap.stories };
    await savePrd(id, JSON.stringify(prd, null, 2)).catch(() => {});
  }
  async function handleNext() {
    if (!id) return;
    await persistStoriesToDisk();
    const state = useWizardStore.getState();
    const draft = { version: 1, projectId: id, currentStep: "configure", describe: { name: state.projectData.name, description: state.projectData.description, workingDirectory: state.projectData.workingDirectory, planAgent: state.projectData.planAgent, planModel: state.projectData.planModel, planEffort: state.projectData.planEffort }, plan: { completed: state.planComplete }, atomize: { storiesCount: state.stories.length }, configure: state.config };
    await saveDraft(id, JSON.stringify(draft, null, 2)).catch(() => {});
    advanceStep(4);
    navigate(`/new/configure/${id}`);
  }
  async function confirmDiscardDraft() {
    if (!id) return;
    await discardDraft(id).catch(() => {});
    navigate("/");
  }

  return (
    <div className="flex h-full min-h-0 gap-4 p-4">
      <Card className="w-72 shrink-0">
        <CardHeader><CardTitle>Queue Sequence</CardTitle></CardHeader>
        <CardContent className="space-y-4">
          <Progress value={pipelinePercent} label="System Health" valueLabel={`${pipelinePercent}%`} />
          <ScrollArea className="max-h-48"><ScrollViewport className="h-full"><ScrollContent className="space-y-2">{stages.map((stage) => <div key={stage.number} className="flex items-center justify-between"><span className="text-xs text-text-muted">{stage.label}</span><Badge variant={STAGE_BADGE[stage.status]} className={`min-w-[4rem] justify-center text-center${stage.status === "running" ? " animate-pulse" : ""}`}>{STAGE_LABEL[stage.status]}</Badge></div>)}</ScrollContent></ScrollViewport></ScrollArea>
          {stageMessage ? <p className="text-xs text-text-muted">{stageMessage}{isRunning ? ` · ${formatElapsed(elapsedSeconds)}` : ""}</p> : null}
          {atomizeError ? <p className="text-xs text-blocked">{atomizeError}</p> : null}
        </CardContent>
      </Card>
      <Card className="flex min-w-0 flex-1 flex-col overflow-hidden">
        <CardHeader className="flex-row items-center justify-between gap-3">
          <CardTitle>Atomic Blueprint</CardTitle>
          <div className="flex items-center gap-2">
            <Badge variant="neutral">{stories.length} stories</Badge>
            <Badge variant="neutral">{totalHours}h est</Badge>
            <Button variant="outline" size="sm" disabled={atomizeStarted && !isDone} onClick={() => addStory(makeBlankStory(stories.length))}>Add story</Button>
          </div>
        </CardHeader>
        <CardContent className="min-h-0 flex-1 p-0">
          <AtomizeStoryList
            stories={stories}
            atomizeStarted={atomizeStarted}
            atomizeError={atomizeError}
            onUpdateStory={updateStory}
            onRequestRemoveStory={setStoryToRemove}
            onDragStart={handleDragStart}
            onDragOver={handleDragOver}
            onDrop={handleDrop}
          />
        </CardContent>
        <CardFooter className="justify-between">
          <Button variant="secondary" size="sm" onClick={() => setDiscardOpen(true)}>Discard draft</Button>
          <span className={`text-xs text-text-muted${isRunning ? " animate-pulse" : ""}`}>{isDone && stories.length > 0 ? `Atomization Complete · ${stories.length} Stories · ${totalHours}h` : `Processing · ${pipelinePercent}% · ${formatElapsed(elapsedSeconds)}`}</span>
          <Button variant="primary" onClick={handleNext} disabled={!isDone && stories.length === 0}>Proceed to validation</Button>
        </CardFooter>
      </Card>
      <AtomizeConfirmationDialog open={discardOpen} onOpenChange={setDiscardOpen} title="Discard this draft?" description="Plan and PRD artifacts will be deleted." confirmLabel="Discard draft" onConfirm={confirmDiscardDraft} />
      <AtomizeConfirmationDialog
        open={Boolean(storyToRemove)}
        onOpenChange={(open) => { if (!open) setStoryToRemove(null); }}
        title="Remove story?"
        description={storyToRemove ? `${storyToRemove.id} will be removed from the atomization list.` : "This story will be removed from the atomization list."}
        confirmLabel="Remove story"
        onConfirm={() => { if (!storyToRemove) return; removeStory(storyToRemove.id); setStoryToRemove(null); }}
      />
    </div>
  );
}
