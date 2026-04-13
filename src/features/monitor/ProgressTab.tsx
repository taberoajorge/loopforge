import { ListChecks } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
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
import { reportError } from "../../lib/reportError";
import { getProjectStories, type IterationStory, onStoriesUpdated } from "../../lib/tauri";
import { useDisplayVocabularyStore } from "../../stores/displayVocabularyStore";
import { TerminalFrame } from "./components/TerminalFrame";

export function ProgressTab({ projectId }: { projectId: string }) {
  const [stories, setStories] = useState<IterationStory[]>([]);
  const vocabulary = useDisplayVocabularyStore((state) => state.vocabulary);

  const refresh = useCallback(() => {
    getProjectStories(projectId)
      .then(setStories)
      .catch((caughtError: unknown) => {
        reportError("ProgressTab.getProjectStories", caughtError);
      });
  }, [projectId]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  useEffect(() => {
    const unlisten = onStoriesUpdated((payload) => {
      if (payload.projectId === projectId) refresh();
    });
    return () => {
      unlisten.then((off) => off());
    };
  }, [projectId, refresh]);

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
          <TableHeader className="sticky top-0 z-10 border-border border-b">
            <TableRow interactive={false}>
              <TableHead className="border-border/40 border-r">ID</TableHead>
              <TableHead className="border-border/40 border-r">Title</TableHead>
              <TableHead className="border-border/40 border-r">Status</TableHead>
              <TableHead className="border-border/40 border-r">Time</TableHead>
              <TableHead numeric>Tries</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {stories.map((story) => (
              <TableRow key={story.id}>
                <TableCell className="border-border/30 border-r text-text-dim text-xs">
                  <span className="block truncate" title={story.id}>
                    {story.id}
                  </span>
                </TableCell>
                <TableCell className="max-w-0 border-border/30 border-r text-sm text-text">
                  <span className="block truncate" title={story.title}>
                    {story.title}
                  </span>
                </TableCell>
                <TableCell className="border-border/30 border-r font-sans">
                  <Badge
                    variant={
                      (vocabulary?.storyStatusVariants[story.status] ?? "neutral") as
                        | "neutral"
                        | "info"
                        | "success"
                        | "warning"
                        | "danger"
                    }
                    className={
                      story.status === "pending"
                        ? "border-border/40 bg-transparent text-text-dim"
                        : ""
                    }
                  >
                    {story.status}
                  </Badge>
                </TableCell>
                <TableCell className="whitespace-nowrap border-border/30 border-r text-text-dim text-xs">
                  {story.durationLabel ?? "—"}
                </TableCell>
                <TableCell className="whitespace-nowrap text-text-dim text-xs" numeric>
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
