import { useEffect, useRef, useState, type DragEvent } from "react";
import { useNavigate, useParams } from "react-router";
import { advanceWizardStep, detectAgents, getAgentCapabilities, getDefaultConfig, saveConfig, saveWizardDraft, validateProjectConfig } from "../../lib/tauri";
import type { AgentCapabilities } from "../../lib/tauri";
import { useAgentStore } from "../../stores/agentStore";
import { useWizardStore } from "../../stores/wizardStore";
import type { ConfigureErrors } from "./configureValidation";
import { ConfigureForm } from "./components/ConfigureForm";

export function Configure() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const agents = useAgentStore((state) => state.agents);
  const setAgents = useAgentStore((state) => state.setAgents);
  const setDetecting = useAgentStore((state) => state.setDetecting);
  const config = useWizardStore((state) => state.config);
  const configLoaded = useWizardStore((state) => state.configLoaded);
  const setConfig = useWizardStore((state) => state.setConfig);
  const setFullConfig = useWizardStore((state) => state.setFullConfig);
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
  const [errors, setErrors] = useState<ConfigureErrors>({});
  const [newAgent, setNewAgent] = useState("");
  const dragIndexRef = useRef<number | null>(null);
  const allAgentNames = agents.map((agent) => agent.name);
  const agentsNotInChain = allAgentNames.filter((agentName) => !fallbackChain.includes(agentName));

  useEffect(() => {
    if (configLoaded) return;
    let cancelled = false;
    getDefaultConfig()
      .then((response) => {
        if (cancelled) return;
        const defaults = response.config;
        setFullConfig(defaults as import("../../types/wizard").WizardConfig);
        setExecuteAgent(defaults.executeAgent);
        setExecuteModel(defaults.executeModel ?? null);
        setExecuteEffort(defaults.executeEffort ?? null);
        setFallbackChain(defaults.fallbackChain);
        setGutterThreshold(defaults.gutterThreshold);
        setMaxIterations(defaults.maxIterations);
        setCooldownSeconds(defaults.cooldownSeconds);
        setTestCommand(defaults.testCommand);
        setMaxVerificationRetries(defaults.maxVerificationRetries);
        setScmProvider(defaults.scmProvider as typeof config.scmProvider);
        setReviewPollingInterval(defaults.reviewPollingInterval);
        setReviewTimeout(defaults.reviewTimeout);
      })
      .catch(() => {});
    return () => { cancelled = true; };
  }, [configLoaded, setFullConfig]);

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
    const sanitizedFallbackChain = fallbackChain.filter((agentName) => agentName !== executeAgent);
    const configPayload = {
      schemaVersion: 1,
      executeAgent,
      executeModel,
      executeEffort,
      fallbackChain: sanitizedFallbackChain,
      gutterThreshold,
      maxIterations,
      cooldownSeconds,
      testCommand,
      maxVerificationRetries,
      scmProvider,
      reviewPollingInterval,
      reviewTimeout,
    };
    try {
      const validationResult = await validateProjectConfig(JSON.stringify(configPayload));
      if (Object.keys(validationResult.errors).length > 0) {
        setErrors(validationResult.errors as ConfigureErrors);
        return;
      }
    } catch {
      return;
    }
    setErrors({});
    await saveConfig(id, JSON.stringify(configPayload, null, 2)).catch(() => {});
    setConfig({
      executeAgent,
      executeModel,
      executeEffort,
      fallbackChain: sanitizedFallbackChain,
      gutterThreshold,
      maxIterations,
      cooldownSeconds,
      testCommand,
      maxVerificationRetries,
      scmProvider,
      reviewPollingInterval,
      reviewTimeout,
    });
    await saveWizardDraft(id, "launch").catch(() => {});
    await advanceWizardStep(5).catch(() => {});
    navigate(`/new/launch/${id}`);
  }

  return (
    <ConfigureForm
      executeAgent={executeAgent}
      executeModel={executeModel}
      executeEffort={executeEffort}
      capabilities={capabilities}
      errors={errors}
      selectableAgentNames={allAgentNames}
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
