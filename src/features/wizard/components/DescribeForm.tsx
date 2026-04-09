import { Button } from "../../../components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../../../components/ui/card";
import { Field } from "../../../components/ui/field";
import { Input } from "../../../components/ui/input";
import { Label } from "../../../components/ui/label";
import { NativeSelect } from "../../../components/ui/native-select";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../../components/ui/select";
import { Textarea } from "../../../components/ui/textarea";
import type { AgentCapabilities, Connection } from "../../../lib/tauri";
type WorkspaceMode = "single" | "connection";
type AgentOption = { name: string; version: string | null; installed: boolean };
type DescribeFormProps = {
  name: string;
  description: string;
  workingDirectory: string;
  workspaceMode: WorkspaceMode;
  selectedConnectionId: string;
  planAgent: string;
  planModel: string | null;
  planEffort: string | null;
  capabilities: AgentCapabilities | null;
  errors: Record<string, string>;
  connections: Connection[];
  allAgents: AgentOption[];
  availableAgentsCount: number;
  submitting: boolean;
  onNameChange: (value: string) => void;
  onDescriptionChange: (value: string) => void;
  onWorkingDirectoryChange: (value: string) => void;
  onWorkspaceModeChange: (mode: WorkspaceMode) => void;
  onConnectionChange: (value: string) => void;
  onBrowseDirectory: () => void;
  onPlanAgentChange: (agentName: string) => void;
  onPlanModelChange: (model: string | null) => void;
  onPlanEffortChange: (effort: string | null) => void;
  onCancel: () => void;
  onNext: () => void;
};
export function DescribeForm(props: DescribeFormProps) {
  const {
    name,
    description,
    workingDirectory,
    workspaceMode,
    selectedConnectionId,
    planAgent,
    planModel,
    planEffort,
    capabilities,
    errors,
    connections,
    allAgents,
    availableAgentsCount,
    submitting,
    onNameChange,
    onDescriptionChange,
    onWorkingDirectoryChange,
    onWorkspaceModeChange,
    onConnectionChange,
    onBrowseDirectory,
    onPlanAgentChange,
    onPlanModelChange,
    onPlanEffortChange,
    onCancel,
    onNext,
  } = props;
  return (
    <form aria-label="Project description form" data-testid="describe-form" className="mx-auto max-w-5xl p-8" onSubmit={(event) => { event.preventDefault(); onNext(); }}>
      <div className="mb-8">
        <h2 className="text-lg font-mono font-bold uppercase tracking-widest text-text">PROJECT IDENTITY</h2>
        <p className="mt-1 text-sm font-sans text-text-muted">Tell the AI what you want to build. The more specific, the better the plan.</p>
      </div>
      <div className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_18rem]">
        <Card>
          <CardContent className="space-y-6 pt-4">
            <Field label="Project Name" errorText={errors.name}>
              <Input value={name} placeholder="e.g. auth-refactor" invalid={Boolean(errors.name)} onChange={(event) => onNameChange(event.target.value)} />
            </Field>
            <div className="space-y-3">
              <Label>Root Directory</Label>
              <div className="flex flex-wrap gap-2">
                <Button variant={workspaceMode === "single" ? "primary" : "secondary"} size="sm" onClick={() => onWorkspaceModeChange("single")}>
                  Single repo
                </Button>
                <Button variant={workspaceMode === "connection" ? "primary" : "secondary"} size="sm" onClick={() => onWorkspaceModeChange("connection")}>
                  Connection
                </Button>
              </div>
              {workspaceMode === "single" ? (
                <Field errorText={errors.workingDirectory}>
                  <div className="flex gap-2">
                    <Input
                      value={workingDirectory}
                      placeholder="/absolute/path/to/project"
                      className="flex-1"
                      invalid={Boolean(errors.workingDirectory)}
                      onChange={(event) => onWorkingDirectoryChange(event.target.value)}
                    />
                    <Button variant="secondary" size="md" data-testid="describe-browse-directory" onClick={onBrowseDirectory}>Browse</Button>
                  </div>
                </Field>
              ) : (
                <Field errorText={errors.workingDirectory}>
                  {connections.length > 0 ? (
                    <NativeSelect value={selectedConnectionId} placeholder="Select a connection..." invalid={Boolean(errors.workingDirectory)} onChange={(event) => onConnectionChange(event.target.value)}>
                      {connections.map((connection) => (
                        <option key={connection.id} value={connection.id}>
                          {connection.name} ({connection.repos.length} repos)
                        </option>
                      ))}
                    </NativeSelect>
                  ) : (
                    <Card variant="ghost" className="border-border/60">
                      <CardContent className="p-3 text-sm font-sans text-text-dim">
                        No connections configured. Create one in Settings first.
                      </CardContent>
                    </Card>
                  )}
                </Field>
              )}
            </div>
            <Field label="Functional Specification" errorText={errors.description}>
              <Textarea
                rows={10}
                value={description}
                invalid={Boolean(errors.description)}
                className="min-h-40"
                placeholder="Describe what you want to build.
Be specific about requirements, constraints, and goals."
                onChange={(event) => onDescriptionChange(event.target.value)}
              />
            </Field>
            {errors.submit ? (
              <Card variant="ghost" className="border-destructive/30 bg-destructive/5">
                <CardContent className="p-3 text-sm font-mono text-destructive">{errors.submit}</CardContent>
              </Card>
            ) : null}
          </CardContent>
        </Card>
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Core plan agent</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              {allAgents.map((agent) => {
                const selected = planAgent === agent.name;
                const unavailable = !agent.installed;
                return (
                  <Button key={agent.name} variant={selected ? "primary" : "secondary"} size="md" fullWidth disabled={unavailable} className="h-auto justify-between py-2" onClick={() => onPlanAgentChange(agent.name)}>
                    <span className="text-sm font-mono">{agent.name}</span>
                    {unavailable ? <span className="text-[10px] font-sans uppercase tracking-widest">Not installed</span> : null}
                  </Button>
                );
              })}
              {capabilities?.supportsModel ? (
                <Field label="Model">
                  <Select value={planModel ?? ""} onValueChange={(val) => onPlanModelChange(val || null)}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>{capabilities.models.map((model) => <SelectItem key={model.id} value={model.id}>{model.label}</SelectItem>)}</SelectContent>
                  </Select>
                </Field>
              ) : null}
              {capabilities?.supportsEffort ? (
                <Field label="Effort">
                  <Select value={planEffort ?? ""} onValueChange={(val) => onPlanEffortChange(val || null)}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>{capabilities.efforts.map((effort) => <SelectItem key={effort.id} value={effort.id}>{effort.label}</SelectItem>)}</SelectContent>
                  </Select>
                </Field>
              ) : null}
            </CardContent>
          </Card>
          <Card variant="elevated">
            <CardHeader>
              <CardTitle>Environment check</CardTitle>
              <CardDescription>Execution readiness snapshot.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-2 text-xs font-mono">
              <div className="flex items-center justify-between">
                <span className="text-text-muted">engine</span>
                <span className="text-success">READY</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-text-muted">agents</span>
                <span className="text-text">{availableAgentsCount} detected</span>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
      <div className="mt-8 flex items-center justify-between border-t border-border pt-6">
        <Button variant="ghost" size="sm" data-testid="describe-cancel-button" onClick={onCancel}>Cancel process</Button>
        <Button variant="primary" size="md" type="submit" data-testid="describe-next-button" disabled={submitting}>{submitting ? "Creating..." : "Next"}</Button>
      </div>
    </form>
  );
}
