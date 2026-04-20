import { useRef, useState } from "react";
import { useNavigate } from "react-router";
import { reportError } from "../lib/reportError";
import {
  advanceWizardStep,
  planUserAction,
  resolvePlanAction,
  savePlan,
  saveWizardDraft,
  startPlan,
} from "../lib/tauri";
import { useWizardStore } from "../stores/wizardStore";

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

    resolvePlanAction(projectId)
      .then((resolved) => {
        if (resolved.action === "resume") {
          useWizardStore.getState().setPlanRunning(true);
        } else if (resolved.action === "prompt_existing") {
          useWizardStore.getState().setPlanContent(resolved.planContent ?? "");
          setShowResumePrompt(true);
        } else {
          void launchPlan();
        }
      })
      .catch((caughtError: unknown) => {
        reportError("usePlanOrchestration.resolvePlanAction", caughtError);
        void launchPlan();
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
    const input = userInput.trim();
    setUserInput("");
    const { planComplete } = useWizardStore.getState();
    if (planComplete) {
      useWizardStore.getState().setPlanRunning(true);
      useWizardStore.getState().setPlanComplete(false);
      useWizardStore.setState({ planEvents: [], planContent: "", stories: [] });
    }
    await planUserAction(projectId, input, "feedback").catch((caughtError: unknown) => {
      reportError("usePlanOrchestration.planUserAction.feedback", caughtError);
      if (planComplete) {
        useWizardStore.getState().setPlanRunning(false);
      }
    });
  }

  async function handleEditToggle() {
    const planContent = useWizardStore.getState().planContent;
    if (isEditing && projectId && editedPlan !== planContent) {
      await savePlan(projectId, editedPlan).catch((caughtError: unknown) => {
        reportError("usePlanOrchestration.savePlan.edit", caughtError);
      });
      useWizardStore.setState({ planContent: editedPlan });
    }
    if (!isEditing) setEditedPlan(planContent);
    setIsEditing((current) => !current);
  }

  async function handleRePlan() {
    if (!projectId) return;
    useWizardStore.getState().setPlanRunning(true);
    useWizardStore.getState().setPlanComplete(false);
    setIsEditing(false);
    setEditedPlan("");
    useWizardStore.setState({ planEvents: [], planContent: "", stories: [] });
    await planUserAction(projectId, "regenerate plan", "replan").catch((caughtError: unknown) => {
      reportError("usePlanOrchestration.planUserAction.replan", caughtError);
      useWizardStore.getState().setPlanRunning(false);
    });
  }

  async function handleNext() {
    if (!projectId) return;
    const { planContent } = useWizardStore.getState();
    const contentToSave = isEditing ? editedPlan : planContent;
    if (contentToSave) {
      await savePlan(projectId, contentToSave).catch((caughtError: unknown) => {
        reportError("usePlanOrchestration.savePlan.next", caughtError);
      });
    }
    await saveWizardDraft(projectId, "atomize").catch((caughtError: unknown) => {
      reportError("usePlanOrchestration.saveWizardDraft.next", caughtError);
    });
    await advanceWizardStep(3).catch((caughtError: unknown) => {
      reportError("usePlanOrchestration.advanceWizardStep.next", caughtError);
    });
    navigate(`/new/atomize/${projectId}`);
  }

  return {
    userInput,
    setUserInput,
    isEditing,
    editedPlan,
    setEditedPlan,
    showResumePrompt,
    initializePlan,
    handleAcceptExisting,
    handleRestartPlan,
    handleSendInput,
    handleEditToggle,
    handleRePlan,
    handleNext,
  };
}
