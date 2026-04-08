import { useCallback, useEffect, useMemo, useState } from "react";
import { Activity } from "lucide-react";
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
import { getIterationHistory, type IterationRow } from "../../lib/tauri";
import { TerminalFrame } from "./components/TerminalFrame";

const RESULT_VARIANT: Record<string, "neutral" | "info" | "success" | "warning" | "danger"> = {
  success: "success",
  pending: "info",
  failed: "danger",
  blocked: "danger",
  skipped: "warning",
};

function formatTime(raw: string): string {
  try {
    const date = new Date(raw.includes("T") ? raw : `${raw}Z`);
    if (Number.isNaN(date.getTime())) return raw;
    return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
  } catch {
    return raw;
  }
}

function formatDuration(secs: number): string {
  if (secs <= 0) return "—";
  if (secs < 60) return `${secs}s`;
  const minutes = Math.floor(secs / 60);
  const remaining = secs % 60;
  return `${minutes}m ${remaining}s`;
}

function liveEventToRow(payload: unknown): IterationRow | null {
  if (!payload || typeof payload !== "object") return null;
  const record = payload as Record<string, unknown>;
  const storyId = typeof record.storyId === "string" ? record.storyId : null;
  if (!storyId) return null;
  return {
    storyId,
    agentUsed: typeof record.agent === "string" ? record.agent : "—",
    result: typeof record.result === "string" ? record.result : "pending",
    durationSecs: typeof record.durationSecs === "number" ? record.durationSecs : 0,
    startedAt: typeof record.timestamp === "string" ? record.timestamp : "",
  };
}

export function ActivityTab({ projectId }: { projectId: string }) {
  const { events } = useProjectEvents(projectId);
  const [dbRows, setDbRows] = useState<IterationRow[]>([]);

  const fetchRows = useCallback(() => {
    getIterationHistory(projectId).then(setDbRows).catch(() => setDbRows([]));
  }, [projectId]);

  useEffect(() => { fetchRows(); }, [fetchRows]);

  useEffect(() => {
    if (events.length > 0) fetchRows();
  }, [events.length, fetchRows]);

  const rows = useMemo(() => {
    const liveRows: IterationRow[] = [];
    for (const event of events) {
      if (event.type !== "iteration_started" && event.type !== "iteration_completed") continue;
      const row = liveEventToRow(event.payload);
      if (row) liveRows.push(row);
    }
    const seen = new Set<string>();
    const merged: IterationRow[] = [];
    for (const row of [...liveRows, ...dbRows]) {
      const key = `${row.storyId}-${row.startedAt}-${row.result}`;
      if (seen.has(key)) continue;
      seen.add(key);
      merged.push(row);
    }
    merged.sort((rowA, rowB) => (rowB.startedAt > rowA.startedAt ? 1 : -1));
    return merged;
  }, [events, dbRows]);

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
          <TableHeader className="sticky top-0 z-10 border-b border-border">
            <TableRow interactive={false}>
              <TableHead className="border-r border-border/40">Story</TableHead>
              <TableHead className="border-r border-border/40">Agent</TableHead>
              <TableHead className="border-r border-border/40">Status</TableHead>
              <TableHead className="border-r border-border/40">Duration</TableHead>
              <TableHead>Time</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {rows.map((row, index) => (
              <TableRow key={`${row.storyId}-${row.startedAt}-${index}`}>
                <TableCell className="border-r border-border/30 text-xs font-mono text-text">
                  <span className="block truncate" title={row.storyId}>{row.storyId}</span>
                </TableCell>
                <TableCell className="border-r border-border/30 text-xs font-mono text-text-dim">
                  <span className="block truncate" title={row.agentUsed}>{row.agentUsed}</span>
                </TableCell>
                <TableCell className="border-r border-border/30">
                  <Badge variant={RESULT_VARIANT[row.result] ?? "neutral"}>{row.result}</Badge>
                </TableCell>
                <TableCell className="border-r border-border/30 text-xs text-text-dim whitespace-nowrap">
                  {formatDuration(row.durationSecs)}
                </TableCell>
                <TableCell className="text-xs font-mono text-text-dim">
                  <span className="block truncate" title={row.startedAt}>{row.startedAt ? formatTime(row.startedAt) : "—"}</span>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>
    </TerminalFrame>
  );
}
