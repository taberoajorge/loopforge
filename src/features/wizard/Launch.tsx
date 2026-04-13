import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { reportError } from "../../lib/reportError";
import { launchProject, validateLaunchReadiness } from "../../lib/tauri";
import { useWizardStore } from "../../stores/wizardStore";
import { LaunchActions } from "./components/LaunchActions";

function SummaryRow({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="flex items-start gap-4 border-border/50 border-b py-2 last:border-0">
      <span className="w-40 shrink-0 pt-0.5 font-mono text-text-muted text-xs uppercase tracking-widest">
        {label}
      </span>
      <span className="flex-1 break-all font-mono text-sm text-text">{value}</span>
    </div>
  );
}

export function Launch() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { projectData, stories, config, reset } = useWizardStore();

  const [launching, setLaunching] = useState(false);
  const [launchError, setLaunchError] = useState<string | null>(null);
  const [readinessIssues, setReadinessIssues] = useState<string[]>([]);
  const [totalHours, setTotalHours] = useState("0.0");

  useEffect(() => {
    if (!id) return;
    validateLaunchReadiness(id)
      .then((result) => {
        setReadinessIssues(result.issues);
        setTotalHours(result.totalEstimatedHours?.toFixed(1) ?? "0.0");
      })
      .catch((caughtError: unknown) => {
        reportError("Launch.validateReadiness", caughtError);
      });
  }, [id]);

  const launchDisabled = readinessIssues.length > 0;

  async function handleLaunch() {
    if (!id || launching || launchDisabled) return;
    setLaunching(true);
    setLaunchError(null);

    try {
      const result = await launchProject(id);
      reset();
      navigate(result.monitorRoute);
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
          <h2 className="font-sans font-semibold text-sm text-text">Launch review</h2>
          <p className="mt-1 font-sans text-text-muted text-xs">
            Review your configuration before starting the loop.
          </p>
        </div>
        <div className="mb-6 grid gap-6 lg:grid-cols-2">
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
              {config.testCommand ? (
                <SummaryRow label="Test command" value={config.testCommand} />
              ) : null}
            </CardContent>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle>Readiness</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <SummaryRow label="Status" value={launchDisabled ? "Needs attention" : "Ready"} />
              {launchDisabled ? (
                <div className="space-y-2">
                  {readinessIssues.map((issue) => (
                    <p key={issue} className="font-sans text-destructive text-sm">
                      {issue}
                    </p>
                  ))}
                </div>
              ) : (
                <p className="font-sans text-sm text-text-muted">
                  All required launch inputs are present.
                </p>
              )}
            </CardContent>
          </Card>
          {launchError ? (
            <Card variant="ghost" className="border-destructive/40 bg-destructive/5">
              <CardContent className="p-3 font-mono text-destructive text-sm">
                Launch failed: {launchError}
              </CardContent>
            </Card>
          ) : null}
        </div>
        <LaunchActions
          launchDisabled={launchDisabled}
          launching={launching}
          onBack={handleBack}
          onCancel={handleBack}
          onLaunch={() => {
            void handleLaunch();
          }}
        />
      </div>
    </div>
  );
}
