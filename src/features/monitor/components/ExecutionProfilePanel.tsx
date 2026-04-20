import { useEffect, useMemo, useRef, useState } from "react";
import { Button } from "../../../components/ui/button";
import { Field } from "../../../components/ui/field";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../../../components/ui/select";
import {
  type AgentCapabilities,
  type ProjectConfig,
  resolveAgentSelection,
  saveConfig,
} from "../../../lib/tauri";
import { useDisplayVocabularyStore } from "../../../stores/displayVocabularyStore";

type ExecutionProfilePanelProps = {
  projectId: string;
  config: ProjectConfig;
  isPaused: boolean;
  onSaved: () => Promise<void>;
};

export function ExecutionProfilePanel({
  projectId,
  config,
  isPaused,
  onSaved,
}: ExecutionProfilePanelProps) {
  const vocabulary = useDisplayVocabularyStore((state) => state.vocabulary);
  const [executeAgent, setExecuteAgent] = useState(config.executeAgent);
  const [executeModel, setExecuteModel] = useState<string | null>(config.executeModel ?? null);
  const [executeEffort, setExecuteEffort] = useState<string | null>(config.executeEffort ?? null);
  const [capabilities, setCapabilities] = useState<AgentCapabilities | null>(null);
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const prevConfigRef = useRef(config);

  useEffect(() => {
    const prev = prevConfigRef.current;
    if (
      prev.executeAgent !== config.executeAgent ||
      prev.executeModel !== config.executeModel ||
      prev.executeEffort !== config.executeEffort
    ) {
      setExecuteAgent(config.executeAgent);
      setExecuteModel(config.executeModel ?? null);
      setExecuteEffort(config.executeEffort ?? null);
    }
    prevConfigRef.current = config;
  }, [config]);

  useEffect(() => {
    let cancelled = false;
    resolveAgentSelection(executeAgent, executeModel, executeEffort)
      .then((result) => {
        if (cancelled) return;
        setCapabilities(result.capabilities);
        setExecuteModel(result.resolvedModel);
        setExecuteEffort(result.resolvedEffort);
      })
      .catch(() => setCapabilities(null));
    return () => {
      cancelled = true;
    };
  }, [executeAgent, executeModel, executeEffort]);

  const dirty = useMemo(() => {
    return (
      executeAgent !== config.executeAgent ||
      executeModel !== (config.executeModel ?? null) ||
      executeEffort !== (config.executeEffort ?? null)
    );
  }, [config, executeAgent, executeModel, executeEffort]);

  async function handleSave() {
    setSaving(true);
    setSaveError(null);
    try {
      const nextConfig: ProjectConfig = {
        ...config,
        executeAgent,
        executeModel,
        executeEffort,
      };
      await saveConfig(projectId, JSON.stringify(nextConfig, null, 2));
      await onSaved();
    } catch (errorValue: unknown) {
      setSaveError(errorValue instanceof Error ? errorValue.message : String(errorValue));
    } finally {
      setSaving(false);
    }
  }

  const showModel = capabilities?.supportsModel;
  const showEffort = capabilities?.supportsEffort;
  const loading = capabilities === null;
  const agentNames = vocabulary?.agentNames.length ? vocabulary.agentNames : [executeAgent];

  return (
    <section
      className="px-6 py-4"
      aria-label="Execution profile"
      data-testid="execution-profile-panel"
    >
      <div className="grid grid-cols-[11rem_1fr_8rem_auto] items-end gap-3">
        <Field label="Execute agent">
          <Select value={executeAgent} onValueChange={setExecuteAgent} disabled={!isPaused}>
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {agentNames.map((agentName) => (
                <SelectItem key={agentName} value={agentName}>
                  {agentName}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
        <Field label="Model" className={showModel ? "" : "invisible"}>
          <Select
            value={executeModel ?? ""}
            onValueChange={(val) => setExecuteModel(val || null)}
            disabled={!isPaused || loading}
          >
            <SelectTrigger>
              <SelectValue placeholder={loading ? "Loading..." : "—"} />
            </SelectTrigger>
            <SelectContent>
              {(capabilities?.models ?? []).map((model) => (
                <SelectItem key={model.id} value={model.id}>
                  {model.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
        <Field label="Effort" className={showEffort ? "" : "invisible"}>
          <Select
            value={executeEffort ?? ""}
            onValueChange={(val) => setExecuteEffort(val || null)}
            disabled={!isPaused || loading}
          >
            <SelectTrigger>
              <SelectValue placeholder={loading ? "Loading..." : "—"} />
            </SelectTrigger>
            <SelectContent>
              {(capabilities?.efforts ?? []).map((effort) => (
                <SelectItem key={effort.id} value={effort.id}>
                  {effort.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
        <Button
          variant="secondary"
          size="md"
          data-testid="execution-profile-save-button"
          disabled={!isPaused || !dirty || saving}
          onClick={() => {
            void handleSave();
          }}
        >
          {saving ? "Saving..." : "Save profile"}
        </Button>
      </div>
      {saveError ? <p className="mt-2 font-sans text-blocked text-xs">{saveError}</p> : null}
      {!isPaused ? (
        <p className="mt-2 font-sans text-text-dim text-xs">
          Pause loop to edit execution profile, then resume to apply.
        </p>
      ) : null}
    </section>
  );
}
