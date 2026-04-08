import { useEffect, useState } from "react";
import { ListChecks } from "lucide-react";
import { EmptyState } from "../../components/EmptyState";
import { Badge } from "../../components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableHeader,
  TableRow,
} from "../../components/ui/table";
import {
  getProjectStories,
  onIterationCompleted,
  onIterationStarted,
  onSessionEnded,
  type IterationStory,
} from "../../lib/tauri";
import { TerminalFrame } from "./components/TerminalFrame";

const STORY_STATUS_VARIANT: Record<
  IterationStory["status"],
  "neutral" | "info" | "success" | "warning" | "danger"
> = {
  completed: "success",
  current: "info",
  blocked: "danger",
  pending: "neutral",
};

function formatDuration(secs?: number): string {
  if (typeof secs !== "number") {
    return "—";
  }
  if (secs <= 0) {
    return "0s";
  }
  if (secs < 60) {
    return `${secs}s`;
  }
  const minutes = Math.floor(secs / 60);
  const remaining = secs % 60;
  return `${minutes}m ${remaining}s`;
}

export function ProgressTab({ projectId }: { projectId: string }) {
  const [stories, setStories] = useState<IterationStory[]>([]);
  const [currentStoryId, setCurrentStoryId] = useState<string | null>(null);

  useEffect(() => {
    getProjectStories(projectId).then(setStories).catch(() => {});
    setCurrentStoryId(null);
  }, [projectId]);

  useEffect(() => {
    const refresh = () => {
      getProjectStories(projectId).then(setStories).catch(() => {});
    };
    const listeners = [
      onIterationStarted((payload: unknown) => {
        const event = payload as { projectId?: string; storyId?: string };
        if (event.projectId !== projectId || typeof event.storyId !== "string") {
          return;
        }
        setCurrentStoryId(event.storyId);
        setStories((current) =>
          current.map((story) => {
            if (story.status === "current" && story.id !== event.storyId) {
              return { ...story, status: "pending" };
            }
            if (story.id === event.storyId && story.status === "pending") {
              return { ...story, status: "current" };
            }
            return story;
          }),
        );
      }),
      onIterationCompleted((payload: unknown) => {
        const event = payload as { projectId?: string; storyId?: string };
        if (event.projectId !== projectId) {
          return;
        }
        if (event.storyId === currentStoryId) {
          setCurrentStoryId(null);
        }
        refresh();
      }),
      onSessionEnded((payload: unknown) => {
        const event = payload as { projectId?: string };
        if (event.projectId !== projectId) {
          return;
        }
        setCurrentStoryId(null);
        refresh();
      }),
    ];
    return () => {
      listeners.forEach((pending) => {
        pending.then((off) => off());
      });
    };
  }, [currentStoryId, projectId]);

  if (stories.length === 0) {
    return (
      <TerminalFrame title="Progress" subtitle="Story execution status">
        <div className="flex h-full items-center justify-center">
          <EmptyState
            title="No stories available"
            description="Run atomization to generate stories for this project."
            icon={<ListChecks className="h-4 w-4" />}
            compact
            padding="tight"
          />
        </div>
      </TerminalFrame>
    );
  }

  return (
    <TerminalFrame title="Progress" subtitle="Story execution status">
      <TableContainer className="h-full">
        <Table className="w-full">
          <colgroup>
            <col className="w-16" />
            <col />
            <col className="w-24" />
            <col className="w-20" />
            <col className="w-14" />
          </colgroup>
          <TableHeader className="sticky top-0 z-10 border-b border-border">
            <TableRow interactive={false}>
              <TableHead className="border-r border-border/40">ID</TableHead>
              <TableHead className="border-r border-border/40">Title</TableHead>
              <TableHead className="border-r border-border/40">Status</TableHead>
              <TableHead className="border-r border-border/40">Time</TableHead>
              <TableHead numeric>Tries</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {stories.map((story) => (
              <TableRow key={story.id}>
                <TableCell className="border-r border-border/30 text-xs text-text-dim">
                  <span className="block truncate" title={story.id}>{story.id}</span>
                </TableCell>
                <TableCell className="border-r border-border/30 text-sm text-text max-w-0">
                  <span className="block truncate" title={story.title}>{story.title}</span>
                </TableCell>
                <TableCell className="border-r border-border/30 font-sans">
                  <Badge
                    variant={STORY_STATUS_VARIANT[story.status]}
                    className={story.status === "pending" ? "text-text-dim border-border/40 bg-transparent" : ""}
                  >
                    {story.status}
                  </Badge>
                </TableCell>
                <TableCell className="border-r border-border/30 text-xs text-text-dim whitespace-nowrap">{formatDuration(story.durationSecs)}</TableCell>
                <TableCell className="text-xs text-text-dim whitespace-nowrap" numeric>
                  {typeof story.attempts === "number" ? story.attempts : "—"}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>
    </TerminalFrame>
  );
}
