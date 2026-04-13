import { Link } from "react-router";
import { StatusBadge } from "@/components/StatusBadge";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import type { Project } from "@/stores/projectStore";

export function ActiveLoopCard({ project }: { project: Project }) {
  const completedStories = project.storiesCompleted ?? 0;
  const totalStories = project.totalStories ?? 0;
  const uptime = project.uptimeLabel ?? project.durationLabel ?? null;

  return (
    <Link to={`/monitor/${project.id}`} className="group block">
      <Card className="transition-colors hover:border-primary/30">
        <CardHeader className="gap-3 border-b-0">
          <div className="flex items-start justify-between gap-3">
            <div className="min-w-0 space-y-1">
              <CardTitle
                className="truncate font-mono transition-colors group-hover:text-primary"
                title={project.name}
              >
                {project.name}
              </CardTitle>
              <CardDescription className="truncate" title={project.description}>
                {project.description}
              </CardDescription>
            </div>
            <div className="flex shrink-0 flex-col items-end gap-1">
              {project.currentAgent ? (
                <Badge variant="info" className="font-mono">
                  {project.currentAgent}
                </Badge>
              ) : null}
              {project.status === "paused" ? <StatusBadge status="paused" /> : null}
              {project.status === "blocked" ? <StatusBadge status="blocked" /> : null}
            </div>
          </div>
        </CardHeader>
        <CardContent className="space-y-2 border-border/60 border-t px-4 py-3">
          <Progress
            value={completedStories}
            max={totalStories}
            showValue
            size="sm"
            valueLabel={`${completedStories}/${totalStories}`}
          />
          <p className="font-mono text-text-dim text-xs">
            {uptime ? `UPTIME: ${uptime}` : "UPTIME: awaiting session telemetry"}
          </p>
        </CardContent>
      </Card>
    </Link>
  );
}
