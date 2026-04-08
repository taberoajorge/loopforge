import { useState } from "react";
import { useNavigate, useParams } from "react-router";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { useWizardStore } from "../../stores/wizardStore";
import { startLoop, finalizeDraft } from "../../lib/tauri";
import { LaunchActions } from "./components/LaunchActions";

function SummaryRow({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="flex items-start gap-4 py-2 border-b border-border/50 last:border-0">
      <span className="text-xs font-mono text-text-muted uppercase tracking-widest w-40 shrink-0 pt-0.5">
        {label}
      </span>
      <span className="text-sm font-mono text-text flex-1 break-all">{value}</span>
    </div>
  );
}

export function Launch() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { projectData, stories, config, reset } = useWizardStore();

  const [launching, setLaunching] = useState(false);
  const [launchError, setLaunchError] = useState<string | null>(null);

  const totalMinutes = stories.reduce((sum, story) => sum + story.estimatedMinutes, 0);
  const totalHours = (totalMinutes / 60).toFixed(1);

  async function handleLaunch() {
    if (!id || launching) return;
    setLaunching(true);
    setLaunchError(null);

    try {
      const projectId = id;

      await finalizeDraft(projectId);

      await startLoop({
        projectId,
      });

      reset();
      navigate(`/monitor/${projectId}`);
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err);
      setLaunchError(message);
      setLaunching(false);
    }
  }

  function handleBack() {
    navigate(-1);
  }

  return (
    <div className="h-full overflow-y-auto p-6">
      <div className="mx-auto max-w-4xl">
        <div className="mb-6">
          <h2 className="text-sm font-sans font-semibold text-text">Launch review</h2>
          <p className="text-xs text-text-muted mt-1 font-sans">Review your configuration before starting the loop.</p>
        </div>
        <div className="grid gap-6 lg:grid-cols-2 mb-6">
          <Card>
            <CardHeader>
              <CardTitle>Project</CardTitle>
            </CardHeader>
            <CardContent>
              <SummaryRow label="Name" value={projectData.name || "—"} />
              <SummaryRow label="Directory" value={projectData.workingDirectory || "—"} />
              <SummaryRow label="Stories" value={stories.length} />
              <SummaryRow label="Est. duration" value={`~${totalHours}h`} />
            </CardContent>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle>Agent config</CardTitle>
            </CardHeader>
            <CardContent>
              <SummaryRow label="Plan agent" value={projectData.planAgent} />
              <SummaryRow label="Plan model" value={projectData.planModel ?? "default"} />
              <SummaryRow label="Execute agent" value={config.executeAgent} />
              <SummaryRow label="Execute model" value={config.executeModel ?? "default"} />
              <SummaryRow label="Fallback" value={config.fallbackChain.join(" → ") || "—"} />
            </CardContent>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle>Loop parameters</CardTitle>
            </CardHeader>
            <CardContent>
              <SummaryRow label="Gutter threshold" value={config.gutterThreshold} />
              <SummaryRow label="Max iterations" value={config.maxIterations} />
              <SummaryRow label="Cooldown" value={`${config.cooldownSeconds}s`} />
              {config.testCommand ? <SummaryRow label="Test command" value={config.testCommand} /> : null}
            </CardContent>
          </Card>
          {launchError ? (
            <Card variant="ghost" className="border-destructive/40 bg-destructive/5">
              <CardContent className="p-3 text-sm font-mono text-destructive">Launch failed: {launchError}</CardContent>
            </Card>
          ) : null}
        </div>
        <LaunchActions launching={launching} onBack={handleBack} onCancel={handleBack} onLaunch={() => { void handleLaunch(); }} />
      </div>
    </div>
  );
}
