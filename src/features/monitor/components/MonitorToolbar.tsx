import { ProjectProgress } from "../../../components/ProjectProgress";
import { StatusBadge, type StatusBadgeStatus } from "../../../components/StatusBadge";
import { useDisplayVocabularyStore } from "../../../stores/displayVocabularyStore";

type MonitorToolbarProps = {
  busyAction: "pause" | "resume" | "stop" | null;
  isPaused: boolean;
  isRunning: boolean;
  onPauseResume: () => void;
  onStop: () => void;
  progress: number;
  status: string;
  storiesDone: number;
  storiesTotal: number;
  storyHint: string | null;
  title: string;
};

export function MonitorToolbar({
  busyAction,
  isPaused,
  isRunning,
  onPauseResume,
  onStop,
  progress,
  status,
  storiesDone,
  storiesTotal,
  storyHint,
  title,
}: MonitorToolbarProps) {
  const vocabulary = useDisplayVocabularyStore((state) => state.vocabulary);
  const badgeStatus = (vocabulary?.monitorStatusBadges[status] ?? "draft") as StatusBadgeStatus;

  return (
    <>
      <div className="flex items-center gap-4 border-border border-b bg-surface px-6 py-3">
        <div className="min-w-0 flex-1">
          <h2 className="truncate font-bold font-mono text-base text-text uppercase tracking-wider">
            SESSION MONITOR: {title}
          </h2>
          <div className="mt-1 flex items-center gap-3 font-sans text-text-muted text-xs">
            <StatusBadge status={badgeStatus} />
            <span>
              {storiesDone}/{storiesTotal} stories
            </span>
            <span>{progress}% complete</span>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={onPauseResume}
            disabled={busyAction !== null || (!isRunning && !isPaused)}
            className="rounded bg-paused/10 px-3 py-1.5 font-sans text-paused text-xs hover:bg-paused/20 disabled:opacity-40"
          >
            {isPaused ? "Resume" : "Pause"}
          </button>
          <button
            type="button"
            onClick={onStop}
            disabled={busyAction !== null || (!isRunning && !isPaused)}
            className="rounded bg-red/10 px-3 py-1.5 font-sans text-red text-xs hover:bg-red/20 disabled:opacity-40"
          >
            Stop
          </button>
        </div>
      </div>
      <div className="border-border border-b bg-surface/30 px-6 py-3">
        <ProjectProgress
          title="Story Progress"
          value={storiesDone}
          total={Math.max(storiesTotal, 1)}
          status={badgeStatus}
          hint={storyHint ?? "No story in progress"}
          progressTone={status === "running" ? "info" : "default"}
          size="sm"
        />
      </div>
    </>
  );
}
