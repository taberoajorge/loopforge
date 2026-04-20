import { Button } from "../../../components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "../../../components/ui/card";
import { Field } from "../../../components/ui/field";
import { Input } from "../../../components/ui/input";
import { NativeSelect } from "../../../components/ui/native-select";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../../../components/ui/select";
import type {
  ConfigureFormHandlers,
  ConfigureFormOptions,
  ConfigureValues,
  ScmProvider,
} from "../configureFormTypes";

type ConfigureFormProps = {
  values: ConfigureValues;
  options: ConfigureFormOptions;
  handlers: ConfigureFormHandlers;
};

export function ConfigureForm(props: ConfigureFormProps) {
  const { values, options, handlers } = props;

  return (
    <div className="h-full overflow-y-auto p-6">
      <div className="mx-auto max-w-5xl">
        <div className="mb-6">
          <h2 className="font-sans font-semibold text-sm text-text">Execution config</h2>
          <p className="mt-1 font-sans text-text-muted text-xs">
            Agents, loop parameters, and review routing for this project.
          </p>
        </div>
        <div className="grid gap-6 lg:grid-cols-2">
          <div className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle>Agent routing</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <Field label="Execute agent" errorText={options.errors.executeAgent}>
                  <Select
                    value={values.executeAgent}
                    onValueChange={(value) => handlers.onValueChange("executeAgent", value)}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {options.selectableAgentNames.map((agentName) => (
                        <SelectItem key={agentName} value={agentName}>
                          {agentName}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </Field>
                {options.capabilities?.supportsModel ? (
                  <Field label="Model">
                    <Select
                      value={values.executeModel ?? ""}
                      onValueChange={(value) =>
                        handlers.onValueChange("executeModel", value || null)
                      }
                    >
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {options.capabilities.models.map((model) => (
                          <SelectItem key={model.id} value={model.id}>
                            {model.label}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </Field>
                ) : null}
                {options.capabilities?.supportsEffort ? (
                  <Field label="Effort">
                    <Select
                      value={values.executeEffort ?? ""}
                      onValueChange={(value) =>
                        handlers.onValueChange("executeEffort", value || null)
                      }
                    >
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {options.capabilities.efforts.map((effort) => (
                          <SelectItem key={effort.id} value={effort.id}>
                            {effort.label}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </Field>
                ) : null}
              </CardContent>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle>Fallback chain</CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                <p className="font-sans text-text-dim text-xs">
                  Agents tried on rate limit or failure. Drag to reorder.
                </p>
                <div className="space-y-2">
                  {values.fallbackChain.map((agentName, index) => (
                    <Card
                      key={agentName}
                      variant="elevated"
                      draggable
                      onDragStart={() => handlers.onDragStart(index)}
                      onDragOver={handlers.onDragOver}
                      onDrop={() => handlers.onDrop(index)}
                      className="cursor-grab active:cursor-grabbing"
                    >
                      <CardContent className="flex items-center gap-3 p-2.5">
                        <span className="w-5 font-mono text-text-dim text-xs">{index + 1}.</span>
                        <span className="flex-1 font-mono text-sm text-text">{agentName}</span>
                        {index === 0 ? (
                          <span className="rounded bg-primary/10 px-1.5 py-0.5 font-sans text-[10px] text-primary">
                            primary
                          </span>
                        ) : null}
                        <Button
                          variant="ghost"
                          size="sm"
                          className="h-6 px-2 text-text-dim text-xs hover:text-destructive"
                          onClick={() => handlers.onRemoveFromChain(agentName)}
                        >
                          Remove
                        </Button>
                      </CardContent>
                    </Card>
                  ))}
                </div>
                {options.agentsNotInChain.length > 0 ? (
                  <div className="flex items-center gap-2">
                    <NativeSelect
                      value={values.newAgent}
                      className="flex-1"
                      placeholder="Add agent..."
                      onChange={(event) => handlers.onValueChange("newAgent", event.target.value)}
                    >
                      {options.agentsNotInChain.map((agentName) => (
                        <option key={agentName} value={agentName}>
                          {agentName}
                        </option>
                      ))}
                    </NativeSelect>
                    <Button
                      variant="secondary"
                      size="sm"
                      disabled={!values.newAgent}
                      onClick={handlers.onAddAgentToChain}
                    >
                      Add
                    </Button>
                  </div>
                ) : null}
              </CardContent>
            </Card>
          </div>
          <div className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle>Loop parameters</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <Field label="Gutter threshold" errorText={options.errors.gutterThreshold}>
                    <Input
                      type="number"
                      min={1}
                      max={20}
                      invalid={Boolean(options.errors.gutterThreshold)}
                      value={values.gutterThreshold}
                      onChange={(event) =>
                        handlers.onValueChange("gutterThreshold", Number(event.target.value))
                      }
                    />
                  </Field>
                  <Field label="Max iterations" errorText={options.errors.maxIterations}>
                    <Input
                      type="number"
                      min={1}
                      max={500}
                      invalid={Boolean(options.errors.maxIterations)}
                      value={values.maxIterations}
                      onChange={(event) =>
                        handlers.onValueChange("maxIterations", Number(event.target.value))
                      }
                    />
                  </Field>
                  <Field label="Cooldown (s)" errorText={options.errors.cooldownSeconds}>
                    <Input
                      type="number"
                      min={0}
                      max={300}
                      invalid={Boolean(options.errors.cooldownSeconds)}
                      value={values.cooldownSeconds}
                      onChange={(event) =>
                        handlers.onValueChange("cooldownSeconds", Number(event.target.value))
                      }
                    />
                  </Field>
                  <Field
                    label="Verification retries"
                    errorText={options.errors.maxVerificationRetries}
                  >
                    <Input
                      type="number"
                      min={1}
                      max={10}
                      invalid={Boolean(options.errors.maxVerificationRetries)}
                      value={values.maxVerificationRetries}
                      onChange={(event) =>
                        handlers.onValueChange("maxVerificationRetries", Number(event.target.value))
                      }
                    />
                  </Field>
                </div>
                <Field label="Test command">
                  <Input
                    value={values.testCommand}
                    placeholder="e.g. npm test"
                    onChange={(event) => handlers.onValueChange("testCommand", event.target.value)}
                  />
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
                  <Select
                    value={values.scmProvider}
                    onValueChange={(value) =>
                      handlers.onValueChange("scmProvider", value as ScmProvider)
                    }
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="auto">Auto-detect</SelectItem>
                      <SelectItem value="github">GitHub</SelectItem>
                      <SelectItem value="gitlab">GitLab</SelectItem>
                      <SelectItem value="none">Disabled</SelectItem>
                    </SelectContent>
                  </Select>
                </Field>
                <div className="grid grid-cols-2 gap-4">
                  <Field label="Poll interval (s)" errorText={options.errors.reviewPollingInterval}>
                    <Input
                      type="number"
                      min={10}
                      max={600}
                      invalid={Boolean(options.errors.reviewPollingInterval)}
                      value={values.reviewPollingInterval}
                      onChange={(event) =>
                        handlers.onValueChange("reviewPollingInterval", Number(event.target.value))
                      }
                    />
                  </Field>
                  <Field label="Timeout (s)" errorText={options.errors.reviewTimeout}>
                    <Input
                      type="number"
                      min={60}
                      max={3600}
                      invalid={Boolean(options.errors.reviewTimeout)}
                      value={values.reviewTimeout}
                      onChange={(event) =>
                        handlers.onValueChange("reviewTimeout", Number(event.target.value))
                      }
                    />
                  </Field>
                </div>
              </CardContent>
            </Card>
            <div className="flex justify-end">
              <Button variant="primary" size="md" onClick={handlers.onNext}>
                Next
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
