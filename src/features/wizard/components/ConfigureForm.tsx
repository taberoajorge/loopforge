import type { DragEvent } from "react";
import { Button } from "../../../components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../../../components/ui/card";
import { Field } from "../../../components/ui/field";
import { Input } from "../../../components/ui/input";
import { NativeSelect } from "../../../components/ui/native-select";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../../components/ui/select";
import type { AgentCapabilities } from "../../../lib/tauri";

type ScmProvider = "auto" | "github" | "gitlab" | "none";

type ConfigureFormProps = {
  executeAgent: string; executeModel: string | null; executeEffort: string | null; capabilities: AgentCapabilities | null; selectableAgentNames: string[]; fallbackChain: string[]; agentsNotInChain: string[]; newAgent: string;
  gutterThreshold: number; maxIterations: number; cooldownSeconds: number; testCommand: string; maxVerificationRetries: number;
  scmProvider: ScmProvider; reviewPollingInterval: number; reviewTimeout: number;
  onExecuteAgentChange: (value: string) => void; onExecuteModelChange: (value: string | null) => void; onExecuteEffortChange: (value: string | null) => void; onNewAgentChange: (value: string) => void; onAddAgentToChain: () => void;
  onRemoveFromChain: (agentName: string) => void; onDragStart: (index: number) => void; onDragOver: (event: DragEvent<HTMLDivElement>) => void; onDrop: (index: number) => void;
  onGutterThresholdChange: (value: number) => void; onMaxIterationsChange: (value: number) => void; onCooldownSecondsChange: (value: number) => void;
  onMaxVerificationRetriesChange: (value: number) => void; onTestCommandChange: (value: string) => void; onScmProviderChange: (value: ScmProvider) => void;
  onReviewPollingIntervalChange: (value: number) => void; onReviewTimeoutChange: (value: number) => void; onNext: () => void;
};

export function ConfigureForm(props: ConfigureFormProps) {
  const {
    executeAgent,
    executeModel,
    executeEffort,
    capabilities,
    selectableAgentNames,
    fallbackChain,
    agentsNotInChain,
    newAgent,
    gutterThreshold,
    maxIterations,
    cooldownSeconds,
    testCommand,
    maxVerificationRetries,
    scmProvider,
    reviewPollingInterval,
    reviewTimeout,
    onExecuteAgentChange,
    onExecuteModelChange,
    onExecuteEffortChange,
    onNewAgentChange,
    onAddAgentToChain,
    onRemoveFromChain,
    onDragStart,
    onDragOver,
    onDrop,
    onGutterThresholdChange,
    onMaxIterationsChange,
    onCooldownSecondsChange,
    onMaxVerificationRetriesChange,
    onTestCommandChange,
    onScmProviderChange,
    onReviewPollingIntervalChange,
    onReviewTimeoutChange,
    onNext,
  } = props;

  return (
    <div className="h-full overflow-y-auto p-6">
      <div className="mx-auto max-w-5xl">
        <div className="mb-6">
          <h2 className="text-sm font-sans font-semibold text-text">Execution config</h2>
          <p className="mt-1 text-xs font-sans text-text-muted">Agents, loop parameters, and review routing for this project.</p>
        </div>
        <div className="grid gap-6 lg:grid-cols-2">
          <div className="space-y-6">
            <Card>
              <CardHeader><CardTitle>Agent routing</CardTitle></CardHeader>
              <CardContent className="space-y-4">
                <Field label="Execute agent">
                  <Select value={executeAgent} onValueChange={onExecuteAgentChange}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>{selectableAgentNames.map((agentName) => <SelectItem key={agentName} value={agentName}>{agentName}</SelectItem>)}</SelectContent>
                  </Select>
                </Field>
                {capabilities?.supportsModel ? (
                  <Field label="Model">
                    <Select value={executeModel ?? ""} onValueChange={(val) => onExecuteModelChange(val || null)}>
                      <SelectTrigger><SelectValue /></SelectTrigger>
                      <SelectContent>{capabilities.models.map((model) => <SelectItem key={model.id} value={model.id}>{model.label}</SelectItem>)}</SelectContent>
                    </Select>
                  </Field>
                ) : null}
                {capabilities?.supportsEffort ? (
                  <Field label="Effort">
                    <Select value={executeEffort ?? ""} onValueChange={(val) => onExecuteEffortChange(val || null)}>
                      <SelectTrigger><SelectValue /></SelectTrigger>
                      <SelectContent>{capabilities.efforts.map((effort) => <SelectItem key={effort.id} value={effort.id}>{effort.label}</SelectItem>)}</SelectContent>
                    </Select>
                  </Field>
                ) : null}
              </CardContent>
            </Card>
            <Card>
              <CardHeader><CardTitle>Fallback chain</CardTitle></CardHeader>
              <CardContent className="space-y-3">
                <p className="text-xs font-sans text-text-dim">Agents tried on rate limit or failure. Drag to reorder.</p>
                <div className="space-y-2">
                  {fallbackChain.map((agentName, index) => (
                    <Card key={agentName} variant="elevated" draggable onDragStart={() => onDragStart(index)} onDragOver={onDragOver} onDrop={() => onDrop(index)} className="cursor-grab active:cursor-grabbing">
                      <CardContent className="flex items-center gap-3 p-2.5">
                        <span className="w-5 text-xs font-mono text-text-dim">{index + 1}.</span>
                        <span className="flex-1 text-sm font-mono text-text">{agentName}</span>
                        {index === 0 ? <span className="rounded bg-primary/10 px-1.5 py-0.5 text-[10px] font-sans text-primary">primary</span> : null}
                        <Button variant="ghost" size="sm" className="h-6 px-2 text-xs text-text-dim hover:text-destructive" onClick={() => onRemoveFromChain(agentName)}>Remove</Button>
                      </CardContent>
                    </Card>
                  ))}
                </div>
                {agentsNotInChain.length > 0 ? (
                  <div className="flex items-center gap-2">
                    <NativeSelect value={newAgent} className="flex-1" placeholder="Add agent..." onChange={(event) => onNewAgentChange(event.target.value)}>
                      {agentsNotInChain.map((agentName) => <option key={agentName} value={agentName}>{agentName}</option>)}
                    </NativeSelect>
                    <Button variant="secondary" size="sm" disabled={!newAgent} onClick={onAddAgentToChain}>Add</Button>
                  </div>
                ) : null}
              </CardContent>
            </Card>
          </div>
          <div className="space-y-6">
            <Card>
              <CardHeader><CardTitle>Loop parameters</CardTitle></CardHeader>
              <CardContent className="space-y-4">
                <div className="grid gap-4 grid-cols-2">
                  <Field label="Gutter threshold">
                    <Input type="number" min={1} max={20} value={gutterThreshold} onChange={(event) => onGutterThresholdChange(Number(event.target.value))} />
                  </Field>
                  <Field label="Max iterations">
                    <Input type="number" min={1} max={500} value={maxIterations} onChange={(event) => onMaxIterationsChange(Number(event.target.value))} />
                  </Field>
                  <Field label="Cooldown (s)">
                    <Input type="number" min={0} max={300} value={cooldownSeconds} onChange={(event) => onCooldownSecondsChange(Number(event.target.value))} />
                  </Field>
                  <Field label="Verification retries">
                    <Input type="number" min={1} max={10} value={maxVerificationRetries} onChange={(event) => onMaxVerificationRetriesChange(Number(event.target.value))} />
                  </Field>
                </div>
                <Field label="Test command">
                  <Input value={testCommand} placeholder="e.g. npm test" onChange={(event) => onTestCommandChange(event.target.value)} />
                </Field>
              </CardContent>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle>Review routing</CardTitle>
                <CardDescription>SCM provider detection and review polling.</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <Field label="SCM provider">
                  <Select value={scmProvider} onValueChange={(val) => onScmProviderChange(val as ScmProvider)}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      <SelectItem value="auto">Auto-detect</SelectItem>
                      <SelectItem value="github">GitHub</SelectItem>
                      <SelectItem value="gitlab">GitLab</SelectItem>
                      <SelectItem value="none">Disabled</SelectItem>
                    </SelectContent>
                  </Select>
                </Field>
                <div className="grid gap-4 grid-cols-2">
                  <Field label="Poll interval (s)">
                    <Input type="number" min={10} max={600} value={reviewPollingInterval} onChange={(event) => onReviewPollingIntervalChange(Number(event.target.value))} />
                  </Field>
                  <Field label="Timeout (s)">
                    <Input type="number" min={60} max={3600} value={reviewTimeout} onChange={(event) => onReviewTimeoutChange(Number(event.target.value))} />
                  </Field>
                </div>
              </CardContent>
            </Card>
            <div className="flex justify-end">
              <Button variant="primary" size="md" onClick={onNext}>Next</Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
