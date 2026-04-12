import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router";
import { advanceWizardStep, createProject, detectAgents, discardDraft, getAgentCapabilities, listConnections, markWizardStale, saveWizardDraft, validateDescribeInput } from "../../lib/tauri";
import type { AgentCapabilities, Connection } from "../../lib/tauri";
import { useAgentStore } from "../../stores/agentStore";
import { useWizardStore } from "../../stores/wizardStore";
import { DescribeForm } from "./components/DescribeForm";

export function Describe() {
  const params = useParams<{ id?: string }>();
  const navigate = useNavigate();
  const agents = useAgentStore((state) => state.agents);
  const setAgents = useAgentStore((state) => state.setAgents);
  const setDetecting = useAgentStore((state) => state.setDetecting);
  const { projectData, projectId: existingProjectId, setProjectData, setProjectId, planContent, reset } = useWizardStore();
  const [name, setName] = useState(projectData.name);
  const [description, setDescription] = useState(projectData.description);
  const [workingDirectory, setWorkingDirectory] = useState(projectData.workingDirectory);
  const [planAgent, setPlanAgent] = useState(projectData.planAgent || "claude");
  const [planModel, setPlanModel] = useState<string | null>(projectData.planModel ?? null);
  const [planEffort, setPlanEffort] = useState<string | null>(projectData.planEffort ?? null);
  const [capabilities, setCapabilities] = useState<AgentCapabilities | null>(null);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [workspaceMode, setWorkspaceMode] = useState<"single" | "connection">("single");
  const [connections, setConnections] = useState<Connection[]>([]);
  const [selectedConnectionId, setSelectedConnectionId] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const draftProjectId = existingProjectId ?? params.id ?? null;
  const availableAgents = agents.filter((agent) => agent.installed);
  const allAgents = agents.length ? agents : [{ name: "claude", version: null, installed: false }, { name: "codex", version: null, installed: false }, { name: "cursor", version: null, installed: false }, { name: "gemini", version: null, installed: false }, { name: "opencode", version: null, installed: false }];

  useEffect(() => {
    listConnections()
      .then(setConnections)
      .catch(() => setConnections([]));
  }, []);

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
    getAgentCapabilities(planAgent)
      .then((nextCapabilities) => {
        if (cancelled) return;
        setCapabilities(nextCapabilities);
        if (nextCapabilities.supportsModel) {
          const keepModel = nextCapabilities.models.some((entry) => entry.id === planModel);
          if (!keepModel) setPlanModel(nextCapabilities.defaultModel ?? nextCapabilities.models[0]?.id ?? null);
        } else setPlanModel(null);
        if (nextCapabilities.supportsEffort) {
          const keepEffort = nextCapabilities.efforts.some((entry) => entry.id === planEffort);
          if (!keepEffort) setPlanEffort(nextCapabilities.defaultEffort ?? nextCapabilities.efforts[0]?.id ?? null);
        } else setPlanEffort(null);
      })
      .catch(() => setCapabilities(null));
    return () => { cancelled = true; };
  }, [planAgent]);

  async function validate(): Promise<Record<string, string>> {
    const nextErrors: Record<string, string> = {};
    if (workspaceMode === "connection" && !selectedConnectionId) {
      nextErrors.workingDirectory = "Select a connection";
      return nextErrors;
    }
    try {
      const input = JSON.stringify({
        name: name.trim(),
        description: description.trim(),
        workingDirectory: workspaceMode === "single" ? workingDirectory.trim() : "placeholder",
        planAgent: planAgent,
      });
      const result = await validateDescribeInput(input);
      return result.errors;
    } catch {
      return nextErrors;
    }
  }

  async function handleCancelProcess() {
    if (draftProjectId) await discardDraft(draftProjectId).catch(() => {});
    reset();
    navigate("/");
  }

  async function handleBrowseDirectory() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selectedDirectory = await open({ directory: true, multiple: false });
      if (selectedDirectory) setWorkingDirectory(selectedDirectory as string);
    } catch {}
  }

  async function handleNext() {
    const validationErrors = await validate();
    if (Object.keys(validationErrors).length > 0) {
      setErrors(validationErrors);
      return;
    }
    setSubmitting(true);
    try {
      let effectiveDirectory = workingDirectory.trim();
      if (workspaceMode === "connection" && selectedConnectionId) {
        const { buildConnectionWorkspace } = await import("../../lib/tauri");
        effectiveDirectory = await buildConnectionWorkspace(selectedConnectionId);
      }
      const updatedData = { name: name.trim(), description: description.trim(), workingDirectory: effectiveDirectory, planAgent, planModel, planEffort };
      setProjectData(updatedData);
      if (existingProjectId) {
        await saveWizardDraft(existingProjectId, "plan").catch(() => {});
        await advanceWizardStep(2).catch(() => {});
        navigate(`/new/plan/${existingProjectId}`);
        return;
      }
      const project = await createProject(updatedData.name, updatedData.description, effectiveDirectory, "describe");
      setProjectId(project.id);
      await saveWizardDraft(project.id, "plan").catch(() => {});
      await advanceWizardStep(2).catch(() => {});
      navigate(`/new/plan/${project.id}`);
    } catch (caughtError: unknown) {
      const errorMessage = caughtError instanceof Error ? caughtError.message : String(caughtError);
      setErrors({ submit: errorMessage });
      setSubmitting(false);
    }
  }

  return (
    <section aria-label="Describe project" data-testid="describe-page">
      <DescribeForm
        name={name}
        description={description}
        workingDirectory={workingDirectory}
        workspaceMode={workspaceMode}
        selectedConnectionId={selectedConnectionId}
        planAgent={planAgent}
        planModel={planModel}
        planEffort={planEffort}
        capabilities={capabilities}
        errors={errors}
        connections={connections}
        allAgents={allAgents}
        availableAgentsCount={availableAgents.length}
        submitting={submitting}
        onNameChange={(value) => { setName(value); setErrors((previousErrors) => ({ ...previousErrors, name: "" })); }}
        onDescriptionChange={(value) => { setDescription(value); setErrors((previousErrors) => ({ ...previousErrors, description: "" })); if (planContent && existingProjectId) { void markWizardStale(existingProjectId, 2); } }}
        onWorkingDirectoryChange={(value) => { setWorkingDirectory(value); setErrors((previousErrors) => ({ ...previousErrors, workingDirectory: "" })); }}
        onWorkspaceModeChange={setWorkspaceMode}
        onConnectionChange={(value) => { setSelectedConnectionId(value); setErrors((previousErrors) => ({ ...previousErrors, workingDirectory: "" })); }}
        onBrowseDirectory={() => { void handleBrowseDirectory(); }}
        onPlanAgentChange={setPlanAgent}
        onPlanModelChange={setPlanModel}
        onPlanEffortChange={setPlanEffort}
        onCancel={() => { void handleCancelProcess(); }}
        onNext={() => { void handleNext(); }}
      />
    </section>
  );
}
