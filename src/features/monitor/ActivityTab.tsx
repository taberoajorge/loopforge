import { Activity } from "lucide-react";
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
import { useProjectEvents } from "../../hooks/useProjectEvents";
import { getActivityFeed, type IterationRow } from "../../lib/tauri";
import { useDisplayVocabularyStore } from "../../stores/displayVocabularyStore";
import { TerminalFrame } from "./components/TerminalFrame";

export function ActivityTab({ projectId }: { projectId: string }) {
  const { events } = useProjectEvents(projectId);
  const [rows, setRows] = useState<IterationRow[]>([]);
  const vocabulary = useDisplayVocabularyStore((state) => state.vocabulary);

  const fetchRows = useCallback(() => {
    getActivityFeed(projectId)
      .then(setRows)
      .catch(() => setRows([]));
  }, [projectId]);

  useEffect(() => {
    fetchRows();
  }, [fetchRows]);

  useEffect(() => {
    if (events.length > 0) fetchRows();
  }, [events.length, fetchRows]);

  if (rows.length === 0) {
    return (
      <TerminalFrame title="Activity" subtitle="Iteration history">
        <div className="flex h-full items-center justify-center">
          <EmptyState
            title="No activity yet"
            description="Iterations appear here once the loop processes stories."
            icon={<Activity className="h-4 w-4" />}
            compact
            padding="tight"
          />
        </div>
      </TerminalFrame>
    );
  }

  return (
    <TerminalFrame title="Activity" subtitle="Iteration history">
      <TableContainer className="h-full">
        <Table className="w-full">
          <colgroup>
            <col className="w-16" />
            <col className="w-24" />
            <col className="w-20" />
            <col className="w-20" />
            <col />
          </colgroup>
          <TableHeader className="sticky top-0 z-10 border-border border-b">
            <TableRow interactive={false}>
              <TableHead className="border-border/40 border-r">Story</TableHead>
              <TableHead className="border-border/40 border-r">Agent</TableHead>
              <TableHead className="border-border/40 border-r">Status</TableHead>
              <TableHead className="border-border/40 border-r">Duration</TableHead>
              <TableHead>Time</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {rows.map((row) => (
              <TableRow key={`${row.storyId}-${row.startedAt}-${row.result}-${row.agentUsed}`}>
                <TableCell className="border-border/30 border-r font-mono text-text text-xs">
                  <span className="block truncate" title={row.storyId}>
                    {row.storyId}
                  </span>
                </TableCell>
                <TableCell className="border-border/30 border-r font-mono text-text-dim text-xs">
                  <span className="block truncate" title={row.agentUsed}>
                    {row.agentUsed}
                  </span>
                </TableCell>
                <TableCell className="border-border/30 border-r">
                  <Badge
                    variant={
                      (vocabulary?.activityResultVariants[row.result] ?? "neutral") as
                        | "neutral"
                        | "info"
                        | "success"
                        | "warning"
                        | "danger"
                    }
                  >
                    {row.result}
                  </Badge>
                </TableCell>
                <TableCell className="whitespace-nowrap border-border/30 border-r text-text-dim text-xs">
                  {row.durationLabel || "—"}
                </TableCell>
                <TableCell className="font-mono text-text-dim text-xs">
                  <span className="block truncate" title={row.startedAt}>
                    {row.timeLabel || "—"}
                  </span>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>
    </TerminalFrame>
  );
}
