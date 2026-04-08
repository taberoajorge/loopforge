import { ProjectProgress } from "../../../components/ProjectProgress";
import { StatusBadge, type StatusBadgeStatus } from "../../../components/StatusBadge";

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

function toBadgeStatus(status: string): StatusBadgeStatus {
  if (status === "ready") return "pending";
  if (status === "running") return "running";
  if (status === "paused") return "paused";
  if (status === "blocked") return "blocked";
  if (status === "failed") return "failed";
  if (status === "completed") return "completed";
  if (status === "archived") return "archived";
  return "draft";
}

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
  return (
    <>
      <div className="bg-surface border-b border-border px-6 py-3 flex items-center gap-4">
        <div className="flex-1 min-w-0">
          <h2 className="text-base font-bold text-text font-mono truncate uppercase tracking-wider">
            SESSION MONITOR: {title}
          </h2>
          <div className="flex items-center gap-3 mt-1 text-xs font-sans text-text-muted">
            <StatusBadge status={toBadgeStatus(status)} />
            <span>{storiesDone}/{storiesTotal} stories</span>
            <span>{progress}% complete</span>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={onPauseResume}
            disabled={busyAction !== null || (!isRunning && !isPaused)}
            className="px-3 py-1.5 rounded bg-paused/10 text-paused text-xs font-sans hover:bg-paused/20 disabled:opacity-40"
          >
            {isPaused ? "Resume" : "Pause"}
          </button>
          <button
            onClick={onStop}
            disabled={busyAction !== null || (!isRunning && !isPaused)}
            className="px-3 py-1.5 rounded bg-red/10 text-red text-xs font-sans hover:bg-red/20 disabled:opacity-40"
          >
            Stop
          </button>
        </div>
      </div>
      <div className="bg-surface/30 border-b border-border px-6 py-3">
        <ProjectProgress
          title="Story Progress"
          value={storiesDone}
          total={Math.max(storiesTotal, 1)}
          status={toBadgeStatus(status)}
          hint={storyHint ?? "No story in progress"}
          progressTone={status === "running" ? "info" : "default"}
          size="sm"
        />
      </div>
    </>
  );
}
