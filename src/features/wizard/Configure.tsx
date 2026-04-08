import { useEffect, useRef, useState, type DragEvent } from "react";
import { useNavigate, useParams } from "react-router";
import { detectAgents, getAgentCapabilities, saveConfig, saveDraft } from "../../lib/tauri";
import type { AgentCapabilities } from "../../lib/tauri";
import { useAgentStore } from "../../stores/agentStore";
import { useWizardStore } from "../../stores/wizardStore";
import { ConfigureForm } from "./components/ConfigureForm";
const KNOWN_AGENTS = ["claude", "codex", "gemini", "opencode", "cursor"];
export function Configure() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const agents = useAgentStore((state) => state.agents);
  const setAgents = useAgentStore((state) => state.setAgents);
  const setDetecting = useAgentStore((state) => state.setDetecting);
  const { config, setConfig, advanceStep } = useWizardStore();
  const [executeAgent, setExecuteAgent] = useState(config.executeAgent);
  const [executeModel, setExecuteModel] = useState<string | null>(config.executeModel ?? null);
  const [executeEffort, setExecuteEffort] = useState<string | null>(config.executeEffort ?? null);
  const [capabilities, setCapabilities] = useState<AgentCapabilities | null>(null);
  const [fallbackChain, setFallbackChain] = useState<string[]>(config.fallbackChain);
  const [gutterThreshold, setGutterThreshold] = useState(config.gutterThreshold);
  const [maxIterations, setMaxIterations] = useState(config.maxIterations);
  const [cooldownSeconds, setCooldownSeconds] = useState(config.cooldownSeconds);
  const [testCommand, setTestCommand] = useState(config.testCommand);
  const [maxVerificationRetries, setMaxVerificationRetries] = useState(config.maxVerificationRetries);
  const [scmProvider, setScmProvider] = useState(config.scmProvider);
  const [reviewPollingInterval, setReviewPollingInterval] = useState(config.reviewPollingInterval);
  const [reviewTimeout, setReviewTimeout] = useState(config.reviewTimeout);
  const [newAgent, setNewAgent] = useState("");
  const dragIndexRef = useRef<number | null>(null);
  const availableAgentNames = agents.filter((agent) => agent.installed).map((agent) => agent.name);
  const selectableAgentNames = availableAgentNames.length ? availableAgentNames : KNOWN_AGENTS;
  const agentsNotInChain = selectableAgentNames.filter((agentName) => !fallbackChain.includes(agentName));

  useEffect(() => {
    let cancelled = false;
    setDetecting(true);
    detectAgents()
      .then((detected) => {
        if (cancelled) return;
        setAgents(
          detected.map((agent) => ({
            name: agent.name,
            installed: agent.available,
            version: agent.version,
          })),
        );
      })
      .catch(() => {
        if (cancelled) return;
        setAgents([]);
      })
      .finally(() => {
        if (!cancelled) setDetecting(false);
      });
    return () => { cancelled = true; };
  }, [setAgents, setDetecting]);
  useEffect(() => {
    let cancelled = false;
    getAgentCapabilities(executeAgent)
      .then((nextCapabilities) => {
        if (cancelled) return;
        setCapabilities(nextCapabilities);
        if (nextCapabilities.supportsModel) {
          const keepModel = nextCapabilities.models.some((entry) => entry.id === executeModel);
          if (!keepModel) setExecuteModel(nextCapabilities.defaultModel ?? nextCapabilities.models[0]?.id ?? null);
        } else setExecuteModel(null);
        if (nextCapabilities.supportsEffort) {
          const keepEffort = nextCapabilities.efforts.some((entry) => entry.id === executeEffort);
          if (!keepEffort) setExecuteEffort(nextCapabilities.defaultEffort ?? nextCapabilities.efforts[0]?.id ?? null);
        } else setExecuteEffort(null);
      })
      .catch(() => setCapabilities(null));
    return () => { cancelled = true; };
  }, [executeAgent]);
  function handleAddToChain() {
    if (!newAgent || fallbackChain.includes(newAgent)) return;
    setFallbackChain((previousChain) => [...previousChain, newAgent]);
    setNewAgent("");
  }
  function handleRemoveFromChain(agentName: string) {
    setFallbackChain((previousChain) => previousChain.filter((existingAgentName) => existingAgentName !== agentName));
  }

  function handleDragStart(index: number) {
    dragIndexRef.current = index;
  }

  function handleDragOver(event: DragEvent<HTMLDivElement>) {
    event.preventDefault();
  }

  function handleDrop(toIndex: number) {
    if (dragIndexRef.current === null || dragIndexRef.current === toIndex) return;
    const reorderedChain = [...fallbackChain];
    const [movedAgent] = reorderedChain.splice(dragIndexRef.current, 1);
    reorderedChain.splice(toIndex, 0, movedAgent);
    setFallbackChain(reorderedChain);
    dragIndexRef.current = null;
  }

  async function handleNext() {
    if (!id) return;
    const configPayload = {
      schemaVersion: 1,
      executeAgent,
      executeModel,
      executeEffort,
      fallbackChain: fallbackChain.filter((agentName) => agentName !== executeAgent),
      gutterThreshold,
      maxIterations,
      cooldownSeconds,
      testCommand,
      maxVerificationRetries,
      scmProvider,
      reviewPollingInterval,
      reviewTimeout,
    };
    await saveConfig(id, JSON.stringify(configPayload, null, 2)).catch(() => {});
    setConfig({
      executeAgent,
      executeModel,
      executeEffort,
      fallbackChain,
      gutterThreshold,
      maxIterations,
      cooldownSeconds,
      testCommand,
      maxVerificationRetries,
      scmProvider,
      reviewPollingInterval,
      reviewTimeout,
    });
    const store = useWizardStore.getState();
    const draft = {
      version: 1,
      projectId: id,
      currentStep: "launch",
      describe: {
        name: store.projectData.name,
        description: store.projectData.description,
        workingDirectory: store.projectData.workingDirectory,
        planAgent: store.projectData.planAgent,
        planModel: store.projectData.planModel,
        planEffort: store.projectData.planEffort,
      },
      plan: { completed: store.planComplete },
      atomize: { storiesCount: store.stories.length },
      configure: { ...configPayload },
    };
    await saveDraft(id, JSON.stringify(draft, null, 2)).catch(() => {});
    advanceStep(5);
    navigate(`/new/launch/${id}`);
  }

  return (
    <ConfigureForm
      executeAgent={executeAgent}
      executeModel={executeModel}
      executeEffort={executeEffort}
      capabilities={capabilities}
      selectableAgentNames={selectableAgentNames}
      fallbackChain={fallbackChain}
      agentsNotInChain={agentsNotInChain}
      newAgent={newAgent}
      gutterThreshold={gutterThreshold}
      maxIterations={maxIterations}
      cooldownSeconds={cooldownSeconds}
      testCommand={testCommand}
      maxVerificationRetries={maxVerificationRetries}
      scmProvider={scmProvider}
      reviewPollingInterval={reviewPollingInterval}
      reviewTimeout={reviewTimeout}
      onExecuteAgentChange={setExecuteAgent}
      onExecuteModelChange={setExecuteModel}
      onExecuteEffortChange={setExecuteEffort}
      onNewAgentChange={setNewAgent}
      onAddAgentToChain={handleAddToChain}
      onRemoveFromChain={handleRemoveFromChain}
      onDragStart={handleDragStart}
      onDragOver={handleDragOver}
      onDrop={handleDrop}
      onGutterThresholdChange={setGutterThreshold}
      onMaxIterationsChange={setMaxIterations}
      onCooldownSecondsChange={setCooldownSeconds}
      onMaxVerificationRetriesChange={setMaxVerificationRetries}
      onTestCommandChange={setTestCommand}
      onScmProviderChange={setScmProvider}
      onReviewPollingIntervalChange={setReviewPollingInterval}
      onReviewTimeoutChange={setReviewTimeout}
      onNext={() => { void handleNext(); }}
    />
  );
}
