import { useRef, useState } from "react";
import { useNavigate } from "react-router";
import { useWizardStore } from "../stores/wizardStore";
import {
  advanceWizardStep, loadExistingPlan, queryPlanStatus, saveWizardDraft,
  savePlan, startPlan, stopPlan, writeToPlan, replan,
} from "../lib/tauri";

export function usePlanOrchestration(projectId: string | undefined) {
  const navigate = useNavigate();
  const projectData = useWizardStore((state) => state.projectData);

  const [userInput, setUserInput] = useState("");
  const [isEditing, setIsEditing] = useState(false);
  const [editedPlan, setEditedPlan] = useState("");
  const [initDone, setInitDone] = useState(false);
  const [showResumePrompt, setShowResumePrompt] = useState(false);
  const launchingRef = useRef(false);

  async function launchPlan() {
    const storeRunning = useWizardStore.getState().planRunning;
    if (!projectId || !projectData.name || storeRunning || launchingRef.current) return;
    launchingRef.current = true;
    useWizardStore.getState().setPlanRunning(true);

    try {
      await startPlan({
        projectId,
        projectDir: projectData.workingDirectory,
        agent: projectData.planAgent,
        model: projectData.planModel,
        effort: projectData.planEffort,
        initialPrompt: projectData.description,
      });
    } catch {
      useWizardStore.getState().setPlanRunning(false);
    } finally {
      launchingRef.current = false;
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
    const { planComplete } = useWizardStore.getState();
    if (planComplete) {
      const feedback = userInput.trim();
      setUserInput("");
      useWizardStore.getState().setPlanRunning(true);
      useWizardStore.getState().setPlanComplete(false);
      useWizardStore.setState({ planEvents: [], planContent: "", stories: [] });
      await replan(projectId, feedback).catch(() => {
        useWizardStore.getState().setPlanRunning(false);
      });
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
    useWizardStore.setState({ planEvents: [], planContent: "", stories: [] });
    void launchPlan();
  }

  async function handleNext() {
    if (!projectId) return;
    const { planContent } = useWizardStore.getState();
    const contentToSave = isEditing ? editedPlan : planContent;
    if (contentToSave) await savePlan(projectId, contentToSave).catch(() => {});
    await saveWizardDraft(projectId, "atomize").catch(() => {});
    await advanceWizardStep(3).catch(() => {});
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
