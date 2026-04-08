import { Link } from "react-router";
import { StatusBadge } from "@/components/StatusBadge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { PROJECT_STATUS_META } from "@/lib/project-status";
import type { Project } from "@/stores/projectStore";

function computeDuration(startedAt: string, endedAt: string): string | null {
  const startedAtMs = Number(new Date(startedAt));
  const endedAtMs = Number(new Date(endedAt));
  if (Number.isNaN(startedAtMs) || Number.isNaN(endedAtMs)) return null;
  const elapsedMs = Math.max(0, endedAtMs - startedAtMs);
  const elapsedHours = Math.floor(elapsedMs / 3600000);
  const elapsedMinutes = Math.floor((elapsedMs % 3600000) / 60000);
  if (elapsedHours > 0) return `${elapsedHours}h ${elapsedMinutes}m`;
  return `${elapsedMinutes}m`;
}

export function CompletedLoopCard({ project }: { project: Project }) {
  const statusMeta = PROJECT_STATUS_META[project.status];
  const completedStories = project.storiesCompleted ?? 0;
  const totalStories = project.totalStories ?? 0;
  const endTime = project.sessionEndedAt ?? project.updatedAt;
  const duration =
    project.sessionStartedAt && endTime
      ? computeDuration(project.sessionStartedAt, endTime)
      : null;

  return (
    <Link to={`/monitor/${project.id}`} className="block">
      <Card className="transition-colors hover:border-border/80">
        <CardHeader className="gap-3 border-b-0">
          <div className="flex items-start justify-between gap-3">
            <div className="min-w-0 space-y-1">
              <CardTitle className="truncate font-mono" title={project.name}>
                {project.name}
              </CardTitle>
              <CardDescription className="truncate" title={project.description}>
                {project.description}
              </CardDescription>
            </div>
            <StatusBadge
              status={statusMeta.badgeStatus}
              label={statusMeta.cardLabel}
            />
          </div>
        </CardHeader>
        <CardContent className="border-t border-border/60 px-4 py-3">
          <div className="flex items-center gap-4 text-xs font-mono text-text-dim">
            <span>{completedStories}/{totalStories} stories</span>
            <span>{duration ?? "Telemetry unavailable"}</span>
          </div>
        </CardContent>
      </Card>
    </Link>
  );
}
