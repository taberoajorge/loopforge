import { useEffect, useRef, useState } from "react";
import {
  getAtomizerPipelineState,
  onAtomizationProgress,
  runAtomizer,
  type AtomizeProgress,
  type Prd,
  type StageStatus,
} from "../lib/tauri";
import { useWizardStore } from "../stores/wizardStore";

export type PipelineStage = { number: number; label: string; status: StageStatus };

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

const PLACEHOLDER_STAGES: PipelineStage[] = [
  { number: 1, label: "Summarize", status: "pending" },
  { number: 2, label: "Chunk", status: "pending" },
  { number: 3, label: "Atomize", status: "pending" },
  { number: 4, label: "Merge", status: "pending" },
];

export function useAtomizerPipeline(projectId: string | undefined) {
  const startedRef = useRef(false);
  const [stages, setStages] = useState<PipelineStage[]>(PLACEHOLDER_STAGES);
  const [atomizeStarted, setAtomizeStarted] = useState(false);
  const [atomizeError, setAtomizeError] = useState<string | null>(null);
  const [stageMessage, setStageMessage] = useState("");
  const [elapsedMs, setElapsedMs] = useState(0);

  useEffect(() => {
    if (!projectId) return;
    const snap = useWizardStore.getState();
    if (!snap.projectData.name) return;

    let disposed = false;

    getAtomizerPipelineState(projectId)
      .then((snapshot) => {
        if (disposed) return;

        if (snapshot && snapshot.stages.length > 0) {
          const backendStages = snapshot.stages.map((stage) => ({
            number: stage.number,
            label: stage.label,
            status: stage.status,
          }));
          const allDone = backendStages.every((stage) => stage.status === "done");
          setAtomizeStarted(true);
          setStages(backendStages);
          setElapsedMs(snapshot.elapsedMs);
          if (snapshot.error) setAtomizeError(snapshot.error);
          if (allDone) {
            startedRef.current = true;
            setStageMessage("Loaded existing pipeline state.");
            return;
          }
        }

        launchAtomizer(snap, disposed);
      })
      .catch(() => {
        if (!disposed) launchAtomizer(snap, disposed);
      });

    function launchAtomizer(wizardSnap: typeof snap, alreadyDisposed: boolean) {
      if (alreadyDisposed || startedRef.current || !projectId) return;
      const resolvedProjectId = projectId;
      startedRef.current = true;
      setAtomizeStarted(true);
      setStageMessage("Summarizing plan...");
      setStages(PLACEHOLDER_STAGES.map((stage) => ({ ...stage, status: stage.number === 1 ? "running" : "pending" })));

      const unlistenPromise = onAtomizationProgress((progress: AtomizeProgress) => {
        if (progress.projectId !== resolvedProjectId) return;
        setStageMessage(progress.message);
        setElapsedMs(progress.elapsedMs);
        setStages((previous) => previous.map((stage) =>
          stage.number === progress.stage
            ? { ...stage, status: "running" }
            : stage.number < progress.stage
              ? { ...stage, status: "done" }
              : stage,
        ));
      });

      runAtomizer({
        projectId: resolvedProjectId,
        projectName: wizardSnap.projectData.name,
        projectDir: wizardSnap.projectData.workingDirectory,
        agent: wizardSnap.projectData.planAgent,
        model: wizardSnap.projectData.planModel,
        effort: wizardSnap.projectData.planEffort,
      }).then((prd: Prd) => {
        if (disposed) return;
        useWizardStore.getState().setStories(prd.stories);
        setStages((previous) => previous.map((stage) => ({ ...stage, status: "done" })));
        setStageMessage(`Done. ${prd.stories.length} stories generated.`);
      }).catch((error: unknown) => {
        if (disposed) return;
        setAtomizeError(error instanceof Error ? error.message : String(error));
        setStages((previous) => previous.map((stage) =>
          stage.status === "running" ? { ...stage, status: "error" } : stage,
        ));
      });

      cleanupRef.current = () => {
        unlistenPromise.then((unlisten) => unlisten());
      };
    }

    const cleanupRef = { current: () => {} };

    return () => {
      disposed = true;
      startedRef.current = false;
      cleanupRef.current();
    };
  }, [projectId]);

  const pipelineProgress = stages.reduce(
    (sum, stage) => sum + (stage.status === "done" ? 1 : stage.status === "running" ? 0.5 : 0),
    0,
  );
  const pipelinePercent = Math.round((pipelineProgress / stages.length) * 100);
  const isDone = stages.every((stage) => stage.status === "done");
  const isRunning = atomizeStarted && !isDone && !atomizeError;

  const elapsedSeconds = Math.floor(elapsedMs / 1000);

  const formatElapsed = (secs: number) =>
    secs < 60 ? `${secs}s` : `${Math.floor(secs / 60)}m ${secs % 60}s`;

  return {
    stages, atomizeStarted, atomizeError, stageMessage,
    pipelinePercent, isDone, isRunning,
    elapsedSeconds, formatElapsed,
  };
}
