import { useEffect, useRef, useState } from "react";
import { onAtomizationProgress, runAtomizer, type AtomizeProgress, type Prd } from "../lib/tauri";
import { useWizardStore } from "../stores/wizardStore";

type StageStatus = "pending" | "running" | "done" | "error";
export type PipelineStage = { number: number; label: string; status: StageStatus };

const INITIAL_STAGES: PipelineStage[] = [
  { number: 1, label: "Summarize", status: "pending" },
  { number: 2, label: "Chunk", status: "pending" },
  { number: 3, label: "Atomize", status: "pending" },
  { number: 4, label: "Merge", status: "pending" },
];

export const STAGE_BADGE: Record<StageStatus, "neutral" | "info" | "success" | "danger"> = {
  pending: "neutral",
  running: "info",
  done: "success",
  error: "danger",
};

export const STAGE_LABEL: Record<StageStatus, string> = {
  pending: "Pending",
  running: "Running",
  done: "Done",
  error: "Error",
};

const atomizerPromiseByProject = new Map<string, Promise<Prd>>();

export function useAtomizerPipeline(projectId: string | undefined) {
  const startedRef = useRef(false);
  const [stages, setStages] = useState(INITIAL_STAGES);
  const [atomizeStarted, setAtomizeStarted] = useState(false);
  const [atomizeError, setAtomizeError] = useState<string | null>(null);
  const [stageMessage, setStageMessage] = useState("");
  const [elapsedSeconds, setElapsedSeconds] = useState(0);

  useEffect(() => {
    if (!projectId) return;
    const snap = useWizardStore.getState();
    if (!snap.projectData.name) return;
    if (snap.stories.length > 0 && snap.projectId === projectId) {
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
    const unlistenPromise = onAtomizationProgress((progress: AtomizeProgress) => {
      if (progress.projectId !== projectId) return;
      setStageMessage(progress.message);
      setStages((previous) => previous.map((stage) =>
        stage.number === progress.stage
          ? { ...stage, status: "running" }
          : stage.number < progress.stage
            ? { ...stage, status: "done" }
            : stage,
      ));
    });
    const releaseListener = () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
    const inFlightPromise = atomizerPromiseByProject.get(projectId) ?? runAtomizer({
      projectId,
      projectName: snap.projectData.name,
      projectDir: snap.projectData.workingDirectory,
      agent: snap.projectData.planAgent,
      model: snap.projectData.planModel,
      effort: snap.projectData.planEffort,
    });
    if (!atomizerPromiseByProject.has(projectId)) {
      atomizerPromiseByProject.set(projectId, inFlightPromise);
    }
    inFlightPromise.then((prd) => {
      if (disposed) return;
      useWizardStore.getState().setStories(prd.stories);
      setStages(INITIAL_STAGES.map((stage) => ({ ...stage, status: "done" })));
      setStageMessage(`Done. ${prd.stories.length} stories generated.`);
    }).catch((error: unknown) => {
      if (disposed) return;
      setAtomizeError(error instanceof Error ? error.message : String(error));
      setStages((previous) => previous.map((stage) =>
        stage.status === "running" ? { ...stage, status: "error" } : stage,
      ));
    }).finally(() => {
      if (atomizerPromiseByProject.get(projectId) === inFlightPromise) {
        atomizerPromiseByProject.delete(projectId);
      }
    });
    return () => {
      disposed = true;
      startedRef.current = false;
      releaseListener();
    };
  }, [projectId]);

  const pipelineProgress = stages.reduce(
    (sum, stage) => sum + (stage.status === "done" ? 1 : stage.status === "running" ? 0.5 : 0),
    0,
  );
  const pipelinePercent = Math.round((pipelineProgress / stages.length) * 100);
  const isDone = stages.every((stage) => stage.status === "done");
  const isRunning = atomizeStarted && !isDone && !atomizeError;

  useEffect(() => {
    if (!isRunning) return;
    setElapsedSeconds(0);
    const timer = setInterval(() => setElapsedSeconds((prev) => prev + 1), 1000);
    return () => clearInterval(timer);
  }, [isRunning]);

  const formatElapsed = (secs: number) =>
    secs < 60 ? `${secs}s` : `${Math.floor(secs / 60)}m ${secs % 60}s`;

  return {
    stages, atomizeStarted, atomizeError, stageMessage,
    pipelinePercent, isDone, isRunning,
    elapsedSeconds, formatElapsed,
  };
}
