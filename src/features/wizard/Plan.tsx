import { useDeferredValue, useEffect, useMemo, useRef, useState } from "react";
import { useNavigate, useParams } from "react-router";
import {
  Badge, Button, Card, CardContent, CardHeader, CardTitle,
  ScrollArea, ScrollViewport, Separator, Textarea,
} from "../../components/ui";
import { MarkdownPreview } from "../../components/MarkdownPreview";
import { useWizardStore } from "../../stores/wizardStore";
import {
  loadExistingPlan, queryPlanStatus, saveDraft,
  savePlan, startPlan, stopPlan, writeToPlan,
} from "../../lib/tauri";
import { usePlanEvents } from "../../hooks/usePlanEvents";
import {
  PlanStreamPanel, shouldRenderPlanEvent,
} from "./components/PlanStreamPanel";
import { PlanWorkspace } from "./components/PlanWorkspace";

export function Plan() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const projectData = useWizardStore((state) => state.projectData);
  const planEvents = useWizardStore((state) => state.planEvents);
  const planContent = useWizardStore((state) => state.planContent);
  const planComplete = useWizardStore((state) => state.planComplete);
  const planRunning = useWizardStore((state) => state.planRunning);
  const appendPlanContent = useWizardStore((state) => state.appendPlanContent);
  const setPlanComplete = useWizardStore((state) => state.setPlanComplete);
  const setPlanRunning = useWizardStore((state) => state.setPlanRunning);
  const advanceStep = useWizardStore((state) => state.advanceStep);

  const [userInput, setUserInput] = useState("");
  const [isEditing, setIsEditing] = useState(false);
  const [editedPlan, setEditedPlan] = useState("");
  const [initDone, setInitDone] = useState(false);
  const [showResumePrompt, setShowResumePrompt] = useState(false);
  const activityEndRef = useRef<HTMLDivElement>(null);
  const feedbackPromptRef = useRef<string | null>(null);

  const { planError, clearError } = usePlanEvents(id);

  const deferredEvents = useDeferredValue(planEvents);
  const visiblePlanEvents = useMemo(
    () => deferredEvents.filter(shouldRenderPlanEvent),
    [deferredEvents],
  );

  useEffect(() => {
    activityEndRef.current?.scrollIntoView({ behavior: "auto" });
  }, [visiblePlanEvents]);

  useEffect(() => {
    if (!id || !projectData.workingDirectory || initDone) return;
    setInitDone(true);
    if (planComplete && planContent.length > 0) return;

    queryPlanStatus(id)
      .then((info) => {
        if (info && info.status === "running") {
          setPlanRunning(true);
          return;
        }
        loadExistingPlan(id)
          .then((existingPlan) => {
            if (!existingPlan) return launchPlan();
            appendPlanContent(existingPlan);
            setShowResumePrompt(true);
          })
          .catch(() => launchPlan());
      })
      .catch(() => {
        loadExistingPlan(id)
          .then((existingPlan) => {
            if (!existingPlan) return launchPlan();
            appendPlanContent(existingPlan);
            setShowResumePrompt(true);
          })
          .catch(() => launchPlan());
      });
  }, [id, projectData.workingDirectory, initDone]);

  async function persistDraft(stepSlug: "plan" | "atomize") {
    if (!id) return;
    const state = useWizardStore.getState();
    const draft = {
      version: 1, projectId: id, currentStep: stepSlug,
      describe: {
        name: state.projectData.name,
        description: state.projectData.description,
        workingDirectory: state.projectData.workingDirectory,
        planAgent: state.projectData.planAgent,
        planModel: state.projectData.planModel,
        planEffort: state.projectData.planEffort,
      },
      plan: { completed: state.planComplete },
      atomize: { storiesCount: state.stories.length },
      configure: state.config,
    };
    await saveDraft(id, JSON.stringify(draft, null, 2)).catch(() => {});
  }

  async function launchPlan() {
    const storeRunning = useWizardStore.getState().planRunning;
    if (!id || !projectData.name || storeRunning) return;
    clearError();
    setPlanRunning(true);

    const effectivePrompt = feedbackPromptRef.current ?? projectData.description;
    feedbackPromptRef.current = null;

    try {
      await startPlan({
        projectId: id,
        projectDir: projectData.workingDirectory,
        agent: projectData.planAgent,
        model: projectData.planModel,
        effort: projectData.planEffort,
        initialPrompt: effectivePrompt,
      });
    } catch {
      setPlanRunning(false);
    }
  }

  function handleAcceptExisting() {
    setShowResumePrompt(false);
    setPlanComplete(true);
  }

  function handleRestartPlan() {
    setShowResumePrompt(false);
    useWizardStore.setState({ planContent: "", stories: [] });
    void launchPlan();
  }

  async function handleSendInput() {
    if (!userInput.trim() || !id) return;
    if (planComplete) {
      const feedback = userInput.trim();
      setUserInput("");
      const currentPlan = useWizardStore.getState().planContent;
      feedbackPromptRef.current =
        `${projectData.description}\n\nPrevious plan:\n${currentPlan}\n\nUser feedback:\n${feedback}`;
      await stopPlan(id).catch(() => {});
      setPlanRunning(false);
      setPlanComplete(false);
      clearError();
      useWizardStore.setState({ planEvents: [], planContent: "", stories: [] });
      void launchPlan();
      return;
    }
    await writeToPlan(id, userInput.trim()).catch(() => {});
    setUserInput("");
  }

  async function handleEditToggle() {
    if (isEditing && id && editedPlan !== planContent) {
      await savePlan(id, editedPlan).catch(() => {});
      useWizardStore.setState({ planContent: editedPlan });
    }
    if (!isEditing) setEditedPlan(planContent);
    setIsEditing((current) => !current);
  }

  async function handleRePlan() {
    if (!id) return;
    await stopPlan(id).catch(() => {});
    setPlanRunning(false);
    setPlanComplete(false);
    setIsEditing(false);
    setEditedPlan("");
    clearError();
    feedbackPromptRef.current = null;
    useWizardStore.setState({ planEvents: [], planContent: "", stories: [] });
    void launchPlan();
  }

  async function handleNext() {
    if (!id) return;
    const contentToSave = isEditing ? editedPlan : planContent;
    if (contentToSave) await savePlan(id, contentToSave).catch(() => {});
    await persistDraft("atomize");
    advanceStep(3);
    navigate(`/new/atomize/${id}`);
  }

  const displayContent = isEditing ? editedPlan : planContent;
  const canEditPlan = planComplete || (showResumePrompt && planContent.length > 0);
  const showPlanPreview =
    isEditing || planComplete || showResumePrompt || (planRunning && planContent.length > 0);

  const previewPanel = (
    <Card className="flex h-full min-h-0 flex-col overflow-hidden rounded-none border-0">
      <CardHeader className="flex-row items-center justify-between p-3">
        <div className="flex items-center gap-2">
          <CardTitle className="text-xs font-sans uppercase tracking-widest text-text-muted">
            Plan Preview
          </CardTitle>
          {planComplete ? (
            <Badge variant="success">Ready</Badge>
          ) : planRunning && planContent.length > 0 ? (
            <Badge variant="info" className="animate-pulse">Streaming</Badge>
          ) : null}
        </div>
        <Button
          variant="ghost" size="sm"
          onClick={handleEditToggle}
          disabled={!canEditPlan && !isEditing}
        >
          {isEditing ? "Preview" : "Edit"}
        </Button>
      </CardHeader>
      <Separator tone="muted" />
      <CardContent className="min-h-0 flex-1 p-3">
        {showPlanPreview && displayContent ? (
          isEditing ? (
            <ScrollArea className="h-full">
              <ScrollViewport className="h-full">
                <Textarea
                  value={editedPlan}
                  onChange={(event) => setEditedPlan(event.target.value)}
                  className="h-full min-h-full resize-none bg-transparent text-xs font-mono"
                />
              </ScrollViewport>
            </ScrollArea>
          ) : (
            <ScrollArea className="h-full">
              <ScrollViewport className="h-full">
                <MarkdownPreview content={displayContent} />
              </ScrollViewport>
            </ScrollArea>
          )
        ) : (
          <Card variant="elevated" className="p-4 text-xs font-mono text-text-dim">
            Plan preview appears when planning is complete.
          </Card>
        )}
      </CardContent>
    </Card>
  );

  return (
    <div className="flex h-full min-h-0 flex-col">
      <PlanWorkspace
        streamPanel={
          <PlanStreamPanel
            activityEvents={visiblePlanEvents}
            allEvents={planEvents}
            planStarted={planRunning}
            planComplete={planComplete}
            showResumePrompt={showResumePrompt}
            planError={planError}
            userInput={userInput}
            onUserInputChange={setUserInput}
            onSendInput={handleSendInput}
            onAcceptExisting={handleAcceptExisting}
            onRestartPlan={handleRestartPlan}
            activityEndRef={activityEndRef}
          />
        }
        previewPanel={previewPanel}
      />
      <Separator />
      <div className="flex shrink-0 items-center justify-between bg-surface/50 px-6 py-3">
        <Button variant="secondary" size="sm" onClick={handleRePlan}>
          Re-plan
        </Button>
        <Button
          onClick={handleNext}
          disabled={!planComplete && planContent.length === 0}
        >
          Next
        </Button>
      </div>
    </div>
  );
}
