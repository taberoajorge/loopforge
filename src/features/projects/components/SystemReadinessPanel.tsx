import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import type { SystemReadiness } from "@/lib/tauri";

type SystemReadinessPanelProps = {
  readiness: SystemReadiness;
  refreshing: boolean;
  onRefresh: () => void;
  onDismiss: () => void;
};

function statusText(ready: boolean, successText: string, errorText: string): string {
  return ready ? successText : errorText;
}

export function SystemReadinessPanel({
  readiness,
  refreshing,
  onRefresh,
  onDismiss,
}: SystemReadinessPanelProps) {
  const availableAgents = readiness.agents.filter((agent) => agent.available);
  const missingAgents = readiness.agents.filter((agent) => !agent.available);
  const hasBlockingIssue =
    !readiness.gitAvailable || !readiness.shellAvailable || availableAgents.length === 0;

  return (
    <Card className="border-warning/40 bg-warning/5">
      <CardContent className="space-y-3 p-4">
        <div className="flex items-start justify-between gap-4">
          <div>
            <h2 className="font-mono text-sm text-text uppercase tracking-wider">System Check</h2>
            <p className="mt-1 text-sm text-text-muted">
              Platform: {readiness.platform}. Resolve missing dependencies before starting loops.
            </p>
          </div>
          <div className="flex items-center gap-2">
            <Button variant="secondary" size="sm" onClick={onRefresh} disabled={refreshing}>
              {refreshing ? "Checking..." : "Re-check"}
            </Button>
            <Button variant="ghost" size="sm" onClick={onDismiss}>
              Dismiss
            </Button>
          </div>
        </div>
        <div className="grid gap-2 sm:grid-cols-3">
          <div className="rounded-md border border-border bg-elevated px-3 py-2 text-xs">
            <p className="font-mono text-text-muted uppercase tracking-wider">Git</p>
            <p className={readiness.gitAvailable ? "text-success" : "text-destructive"}>
              {statusText(readiness.gitAvailable, "Available", "Missing")}
            </p>
          </div>
          <div className="rounded-md border border-border bg-elevated px-3 py-2 text-xs">
            <p className="font-mono text-text-muted uppercase tracking-wider">Shell</p>
            <p className={readiness.shellAvailable ? "text-success" : "text-destructive"}>
              {statusText(readiness.shellAvailable, "Available", "Missing")}
            </p>
          </div>
          <div className="rounded-md border border-border bg-elevated px-3 py-2 text-xs">
            <p className="font-mono text-text-muted uppercase tracking-wider">Agents</p>
            <p className={availableAgents.length > 0 ? "text-success" : "text-destructive"}>
              {availableAgents.length} detected
            </p>
          </div>
        </div>
        {missingAgents.length > 0 ? (
          <p className="text-text-muted text-xs">
            Missing agents: {missingAgents.map((agent) => agent.binary).join(", ")}
          </p>
        ) : null}
        {hasBlockingIssue ? (
          <p className="text-warning text-xs">
            At least one required dependency is missing. Install git and one agent CLI to continue.
          </p>
        ) : (
          <p className="text-success text-xs">Environment is ready.</p>
        )}
      </CardContent>
    </Card>
  );
}
