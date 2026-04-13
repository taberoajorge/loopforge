import { useEffect, useRef, useState } from "react";
import {
  getAtomizerPipelineState,
  onAtomizationProgress,
  type Prd,
  runAtomizer,
  type StageSnapshot,
  type StageStatus,
} from "../lib/tauri";
import { useWizardStore } from "../stores/wizardStore";

export type PipelineStage = { number: number; label: string; status: StageStatus };

const PLACEHOLDER_STAGES: PipelineStage[] = [
  { number: 1, label: "Summarize", status: "pending" },
  { number: 2, label: "Chunk", status: "pending" },
  { number: 3, label: "Atomize", status: "pending" },
  { number: 4, label: "Merge", status: "pending" },
];

function normalizeStages(stages: StageSnapshot[]): PipelineStage[] {
  return stages.map((stage) => ({
    number: stage.number,
    label: stage.label,
    status: stage.status,
  }));
}

export function useAtomizerPipeline(projectId: string | undefined) {
  const startedRef = useRef(false);
  const [stages, setStages] = useState<PipelineStage[]>(PLACEHOLDER_STAGES);
  const [atomizeStarted, setAtomizeStarted] = useState(false);
  const [atomizeError, setAtomizeError] = useState<string | null>(null);
  const [stageMessage, setStageMessage] = useState("");
  const [elapsedMs, setElapsedMs] = useState(0);
  const [pipelinePercent, setPipelinePercent] = useState(0);
  const [isDone, setIsDone] = useState(false);
  const [isRunning, setIsRunning] = useState(false);

  useEffect(() => {
    if (!projectId) return;
    const wizardSnapshot = useWizardStore.getState();
    if (!wizardSnapshot.projectData.name) return;

    let disposed = false;
    let pollingTimer: ReturnType<typeof setInterval> | null = null;
    let progressUnlisten: (() => void) | null = null;

    const syncFromBackend = async () => {
      try {
        const snapshot = await getAtomizerPipelineState(projectId);
        if (disposed || !snapshot) return snapshot;
        const backendStages =
          snapshot.stages.length > 0 ? normalizeStages(snapshot.stages) : PLACEHOLDER_STAGES;
        setStages(backendStages);
        setPipelinePercent(snapshot.percent ?? 0);
        setIsDone(Boolean(snapshot.isDone));
        setIsRunning(Boolean(snapshot.isRunning));
        setElapsedMs(snapshot.elapsedMs ?? 0);
        setAtomizeStarted(snapshot.stages.length > 0 || snapshot.isRunning || snapshot.isDone);
        if (snapshot.error) {
          setAtomizeError(snapshot.error);
        }
        return snapshot;
      } catch {
        return null;
      }
    };

    const startPipeline = async () => {
      if (startedRef.current) return;
      startedRef.current = true;
      setAtomizeStarted(true);
      setAtomizeError(null);
      setStageMessage("Summarizing plan...");
      try {
        const prd: Prd = await runAtomizer({
          projectId,
          projectName: wizardSnapshot.projectData.name,
          projectDir: wizardSnapshot.projectData.workingDirectory,
          agent: wizardSnapshot.projectData.planAgent,
          model: wizardSnapshot.projectData.planModel,
          effort: wizardSnapshot.projectData.planEffort,
        });
        if (disposed) return;
        useWizardStore.getState().setStories(prd.stories);
        setStageMessage(`Done. ${prd.stories.length} stories generated.`);
      } catch (errorValue: unknown) {
        if (disposed) return;
        setAtomizeError(errorValue instanceof Error ? errorValue.message : String(errorValue));
      } finally {
        await syncFromBackend();
      }
    };

    const attachProgressListener = async () => {
      const unlisten = await onAtomizationProgress((progress) => {
        if (progress.projectId !== projectId) return;
        setStageMessage(progress.message);
        setElapsedMs(progress.elapsedMs);
        void syncFromBackend();
      });
      if (!disposed) {
        progressUnlisten = unlisten;
      } else {
        unlisten();
      }
    };

    void (async () => {
      const snapshot = await syncFromBackend();
      await attachProgressListener();
      const shouldLaunch = !snapshot || (!snapshot.isRunning && !snapshot.isDone);
      if (shouldLaunch) {
        await startPipeline();
      }
      pollingTimer = setInterval(() => {
        void syncFromBackend();
      }, 1500);
    })();

    return () => {
      disposed = true;
      startedRef.current = false;
      if (pollingTimer) clearInterval(pollingTimer);
      if (progressUnlisten) progressUnlisten();
    };
  }, [projectId]);

  const elapsedSeconds = Math.floor(elapsedMs / 1000);
  const formatElapsed = (secs: number) =>
    secs < 60 ? `${secs}s` : `${Math.floor(secs / 60)}m ${secs % 60}s`;

  return {
    stages,
    atomizeStarted,
    atomizeError,
    stageMessage,
    pipelinePercent,
    isDone,
    isRunning,
    elapsedSeconds,
    formatElapsed,
  };
}
