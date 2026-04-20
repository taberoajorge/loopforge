import { useDeferredValue, useEffect, useRef } from "react";
import { useParams } from "react-router";
import { MarkdownPreview } from "../../components/MarkdownPreview";
import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { ScrollArea, ScrollViewport } from "../../components/ui/scroll-area";
import { Separator } from "../../components/ui/separator";
import { Textarea } from "../../components/ui/textarea";
import { usePlanEvents } from "../../hooks/usePlanEvents";
import { usePlanOrchestration } from "../../hooks/usePlanOrchestration";
import { useWizardStore } from "../../stores/wizardStore";
import { PlanStreamPanel } from "./components/PlanStreamPanel";
import { PlanWorkspace } from "./components/PlanWorkspace";

export function Plan() {
  const { id } = useParams<{ id: string }>();
  const planEvents = useWizardStore((state) => state.planEvents);
  const planContent = useWizardStore((state) => state.planContent);
  const planComplete = useWizardStore((state) => state.planComplete);
  const planRunning = useWizardStore((state) => state.planRunning);
  const activityEndRef = useRef<HTMLDivElement>(null);

  const { planError } = usePlanEvents(id);
  const orchestration = usePlanOrchestration(id);

  const visiblePlanEvents = useDeferredValue(planEvents);

  useEffect(() => {
    activityEndRef.current?.scrollIntoView({ behavior: "auto" });
  }, []);

  useEffect(() => {
    orchestration.initializePlan();
  }, [orchestration.initializePlan]);

  const displayContent = orchestration.isEditing ? orchestration.editedPlan : planContent;
  const canEditPlan = planComplete || (orchestration.showResumePrompt && planContent.length > 0);
  const showPlanPreview =
    orchestration.isEditing ||
    planComplete ||
    orchestration.showResumePrompt ||
    (planRunning && planContent.length > 0);

  const previewPanel = (
    <Card className="flex h-full min-h-0 flex-col overflow-hidden rounded-none border-0">
      <CardHeader className="flex-row items-center justify-between p-3">
        <div className="flex items-center gap-2">
          <CardTitle className="font-sans text-text-muted text-xs uppercase tracking-widest">
            Plan Preview
          </CardTitle>
          {planComplete ? (
            <Badge variant="success">Ready</Badge>
          ) : planRunning && planContent.length > 0 ? (
            <Badge variant="info" className="animate-pulse">
              Streaming
            </Badge>
          ) : null}
        </div>
        <Button
          variant="ghost"
          size="sm"
          onClick={orchestration.handleEditToggle}
          disabled={!canEditPlan && !orchestration.isEditing}
        >
          {orchestration.isEditing ? "Preview" : "Edit"}
        </Button>
      </CardHeader>
      <Separator tone="muted" />
      <CardContent className="min-h-0 flex-1 p-3">
        {showPlanPreview && displayContent ? (
          orchestration.isEditing ? (
            <ScrollArea className="h-full">
              <ScrollViewport className="h-full">
                <Textarea
                  value={orchestration.editedPlan}
                  onChange={(event) => orchestration.setEditedPlan(event.target.value)}
                  className="h-full min-h-full resize-none bg-transparent font-mono text-xs"
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
          <Card variant="elevated" className="p-4 font-mono text-text-dim text-xs">
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
            showResumePrompt={orchestration.showResumePrompt}
            planError={planError}
            userInput={orchestration.userInput}
            onUserInputChange={orchestration.setUserInput}
            onSendInput={orchestration.handleSendInput}
            onAcceptExisting={orchestration.handleAcceptExisting}
            onRestartPlan={orchestration.handleRestartPlan}
            activityEndRef={activityEndRef}
          />
        }
        previewPanel={previewPanel}
      />
      <Separator />
      <div className="flex shrink-0 items-center justify-between bg-surface/50 px-6 py-3">
        <Button variant="secondary" size="sm" onClick={orchestration.handleRePlan}>
          Re-plan
        </Button>
        <Button
          onClick={orchestration.handleNext}
          disabled={!planComplete && planContent.length === 0}
        >
          Next
        </Button>
      </div>
    </div>
  );
}
