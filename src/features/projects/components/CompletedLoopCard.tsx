import { Link } from "react-router";
import { StatusBadge, type StatusBadgeStatus } from "@/components/StatusBadge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { useDisplayVocabularyStore } from "@/stores/displayVocabularyStore";
import type { Project } from "@/stores/projectStore";

export function CompletedLoopCard({ project }: { project: Project }) {
  const vocabulary = useDisplayVocabularyStore((state) => state.vocabulary);
  const statusMeta = vocabulary?.projectStatusMeta[project.status] ?? {
    badgeStatus: project.status === "active" ? "running" : project.status,
    cardLabel: `${project.status.slice(0, 1).toUpperCase()}${project.status.slice(1)}`,
    sidebarLabel: `${project.status.slice(0, 1).toUpperCase()}${project.status.slice(1)}`,
  };
  const completedStories = project.storiesCompleted ?? 0;
  const totalStories = project.totalStories ?? 0;
  const duration = project.durationLabel ?? null;

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
              status={statusMeta.badgeStatus as StatusBadgeStatus}
              label={statusMeta.cardLabel}
            />
          </div>
        </CardHeader>
        <CardContent className="border-border/60 border-t px-4 py-3">
          <div className="flex items-center gap-4 font-mono text-text-dim text-xs">
            <span>
              {completedStories}/{totalStories} stories
            </span>
            <span>{duration ?? "Telemetry unavailable"}</span>
          </div>
        </CardContent>
      </Card>
    </Link>
  );
}
