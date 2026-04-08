import { useRef, useState } from "react";
import { useNavigate } from "react-router";
import { useWizardStore } from "../stores/wizardStore";
import {
  loadExistingPlan, queryPlanStatus, saveDraft,
  savePlan, startPlan, stopPlan, writeToPlan,
} from "../lib/tauri";
import { buildDraftPayload } from "../lib/draft-payload";

export function usePlanOrchestration(projectId: string | undefined) {
  const navigate = useNavigate();
  const projectData = useWizardStore((state) => state.projectData);
  const advanceStep = useWizardStore((state) => state.advanceStep);

  const [userInput, setUserInput] = useState("");
  const [isEditing, setIsEditing] = useState(false);
  const [editedPlan, setEditedPlan] = useState("");
  const [initDone, setInitDone] = useState(false);
  const [showResumePrompt, setShowResumePrompt] = useState(false);
  const feedbackPromptRef = useRef<string | null>(null);

  async function launchPlan() {
    const storeRunning = useWizardStore.getState().planRunning;
    if (!projectId || !projectData.name || storeRunning) return;
    useWizardStore.getState().setPlanRunning(true);

    const effectivePrompt = feedbackPromptRef.current ?? projectData.description;
    feedbackPromptRef.current = null;

    try {
      await startPlan({
        projectId,
        projectDir: projectData.workingDirectory,
        agent: projectData.planAgent,
        model: projectData.planModel,
        effort: projectData.planEffort,
        initialPrompt: effectivePrompt,
      });
    } catch {
      useWizardStore.getState().setPlanRunning(false);
    }
  }

  function initializePlan() {
    if (!projectId || !projectData.workingDirectory || initDone) return;
    setInitDone(true);
    const { planComplete, planContent } = useWizardStore.getState();
    if (planComplete && planContent.length > 0) return;

    queryPlanStatus(projectId)
      .then((info) => {
        if (info && info.status === "running") {
          useWizardStore.getState().setPlanRunning(true);
          return;
        }
        loadExistingPlan(projectId)
          .then((existingPlan) => {
            if (!existingPlan) return launchPlan();
            useWizardStore.getState().appendPlanContent(existingPlan);
            setShowResumePrompt(true);
          })
          .catch(() => launchPlan());
      })
      .catch(() => {
        loadExistingPlan(projectId)
          .then((existingPlan) => {
            if (!existingPlan) return launchPlan();
            useWizardStore.getState().appendPlanContent(existingPlan);
            setShowResumePrompt(true);
          })
          .catch(() => launchPlan());
      });
  }

  function handleAcceptExisting() {
    setShowResumePrompt(false);
    useWizardStore.getState().setPlanComplete(true);
  }

  function handleRestartPlan() {
    setShowResumePrompt(false);
    useWizardStore.setState({ planContent: "", stories: [] });
    void launchPlan();
  }

  async function handleSendInput() {
    if (!userInput.trim() || !projectId) return;
    const { planComplete, planContent } = useWizardStore.getState();
    if (planComplete) {
      const feedback = userInput.trim();
      setUserInput("");
      feedbackPromptRef.current =
        `${projectData.description}\n\nPrevious plan:\n${planContent}\n\nUser feedback:\n${feedback}`;
      await stopPlan(projectId).catch(() => {});
      useWizardStore.getState().setPlanRunning(false);
      useWizardStore.getState().setPlanComplete(false);
      useWizardStore.setState({ planEvents: [], planContent: "", stories: [] });
      void launchPlan();
      return;
    }
    await writeToPlan(projectId, userInput.trim()).catch(() => {});
    setUserInput("");
  }

  async function handleEditToggle() {
    const planContent = useWizardStore.getState().planContent;
    if (isEditing && projectId && editedPlan !== planContent) {
      await savePlan(projectId, editedPlan).catch(() => {});
      useWizardStore.setState({ planContent: editedPlan });
    }
    if (!isEditing) setEditedPlan(planContent);
    setIsEditing((current) => !current);
  }

  async function handleRePlan() {
    if (!projectId) return;
    await stopPlan(projectId).catch(() => {});
    useWizardStore.getState().setPlanRunning(false);
    useWizardStore.getState().setPlanComplete(false);
    setIsEditing(false);
    setEditedPlan("");
    feedbackPromptRef.current = null;
    useWizardStore.setState({ planEvents: [], planContent: "", stories: [] });
    void launchPlan();
  }

  async function handleNext() {
    if (!projectId) return;
    const { planContent } = useWizardStore.getState();
    const contentToSave = isEditing ? editedPlan : planContent;
    if (contentToSave) await savePlan(projectId, contentToSave).catch(() => {});
    await saveDraft(projectId, buildDraftPayload(projectId, "atomize")).catch(() => {});
    advanceStep(3);
    navigate(`/new/atomize/${projectId}`);
  }

  return {
    userInput, setUserInput,
    isEditing, editedPlan, setEditedPlan,
    showResumePrompt,
    initializePlan,
    handleAcceptExisting, handleRestartPlan,
    handleSendInput, handleEditToggle, handleRePlan, handleNext,
  };
}
