import { Loader2 } from "lucide-react";
import { memo, type RefObject, useMemo } from "react";
import { Badge } from "../../../components/ui/badge";
import { Button } from "../../../components/ui/button";
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from "../../../components/ui/card";
import { Input } from "../../../components/ui/input";
import { ScrollArea, ScrollContent, ScrollViewport } from "../../../components/ui/scroll-area";
import { Separator } from "../../../components/ui/separator";
import { useDisplayVocabularyStore } from "../../../stores/displayVocabularyStore";
import type { PlanEvent } from "../../../types/wizard";
import { usePlanStreamTimers } from "../hooks/usePlanStreamTimers";
import { PlanErrorCard } from "./PlanErrorCard";
import { PlanResumePrompt } from "./PlanResumePrompt";
import { PlanRunningIndicator } from "./PlanRunningIndicator";

type PlanStreamPanelProps = {
  activityEvents: PlanEvent[];
  allEvents: PlanEvent[];
  planStarted: boolean;
  planComplete: boolean;
  showResumePrompt: boolean;
  planError: string | null;
  userInput: string;
  onUserInputChange: (value: string) => void;
  onSendInput: () => void;
  onAcceptExisting: () => void;
  onRestartPlan: () => void;
  activityEndRef: RefObject<HTMLDivElement | null>;
};

const PlanEventRow = memo(function PlanEventRow({
  event,
  kindMeta,
}: {
  event: PlanEvent;
  kindMeta: Record<string, { label: string; variant: string }>;
}) {
  const config = kindMeta[event.kind] ?? { label: "WAIT", variant: "neutral" };
  return (
    <div
      className="flex gap-2 border-border/30 border-b py-1 last:border-0"
      style={{ contentVisibility: "auto", containIntrinsicSize: "auto 28px" }}
    >
      <span className="mt-0.5 shrink-0 font-mono text-text-dim text-xs">{event.timestamp}</span>
      <Badge
        variant={config.variant as "neutral" | "info" | "warning" | "danger" | "success"}
        className="mt-0.5 h-fit rounded-sm px-1 font-mono text-[10px]"
      >
        {config.label}
      </Badge>
      <span className="break-all font-mono text-text text-xs leading-5">{event.content}</span>
    </div>
  );
});

export function PlanStreamPanel(props: PlanStreamPanelProps) {
  const {
    activityEvents,
    allEvents,
    planStarted,
    planComplete,
    showResumePrompt,
    planError,
    userInput,
    onUserInputChange,
    onSendInput,
    onAcceptExisting,
    onRestartPlan,
    activityEndRef,
  } = props;
  const vocabulary = useDisplayVocabularyStore((state) => state.vocabulary);
  const maxVisibleEvents = vocabulary?.maxVisibleActivityEvents ?? 200;

  const isRunning = planStarted && !planComplete;
  const hasSearch = useMemo(() => allEvents.some((event) => event.kind === "search"), [allEvents]);
  const hasDocsLookup = useMemo(
    () => allEvents.some((event) => event.kind === "docsLookup"),
    [allEvents],
  );
  const hasPlanContent = useMemo(
    () => allEvents.some((event) => event.kind === "planContent"),
    [allEvents],
  );

  const cappedEvents = useMemo(() => {
    if (activityEvents.length <= maxVisibleEvents) return activityEvents;
    return activityEvents.slice(-maxVisibleEvents);
  }, [activityEvents, maxVisibleEvents]);
  const { elapsedSeconds, secondsSinceLastEvent } = usePlanStreamTimers({
    isRunning,
    eventCount: allEvents.length,
  });

  const stallThreshold = vocabulary?.stallThresholdSecs ?? 120;
  const isStalled = isRunning && secondsSinceLastEvent > stallThreshold;

  function resolvePhaseLabel(): string {
    if (isStalled) return `No output for ${secondsSinceLastEvent}s — agent may be stalled`;
    if (!planStarted) return "Initializing agent...";
    if (hasPlanContent) return "Writing plan...";
    if (hasDocsLookup) return "Analyzing documentation...";
    if (hasSearch) return "Researching codebase...";
    if (activityEvents.length > 0) return "Agent working...";
    if (elapsedSeconds < 20) return "Connecting to agent...";
    return "Waiting for agent response...";
  }

  function resolveStatusBadge() {
    if (planError) return <Badge variant="danger">Error</Badge>;
    if (planStarted && !planComplete) {
      return (
        <Badge variant="warning" className="animate-pulse">
          Running
        </Badge>
      );
    }
    if (planComplete) return <Badge variant="success">Complete</Badge>;
    return null;
  }

  return (
    <Card
      className="flex h-full min-h-0 flex-col overflow-hidden rounded-none border-0 border-r"
      role="region"
      aria-label="Plan activity stream"
      data-testid="plan-stream-panel"
    >
      <CardHeader className="gap-3 p-3">
        <div className="flex items-center gap-2">
          <CardTitle className="font-mono text-text-muted text-xs uppercase tracking-widest">
            Activity Stream
          </CardTitle>
          {resolveStatusBadge()}
          <div className="ml-auto flex items-center gap-2">
            {hasSearch ? <Badge variant="info">EXA ACTIVE</Badge> : null}
            {hasDocsLookup ? <Badge variant="warning">CONTEXT7 ACTIVE</Badge> : null}
          </div>
        </div>
        <Separator tone="muted" />
      </CardHeader>
      <CardContent className="min-h-0 flex-1 p-0">
        <ScrollArea className="h-full">
          <ScrollViewport padding="md" className="h-full">
            <ScrollContent className="space-y-2">
              {showResumePrompt ? (
                <PlanResumePrompt
                  onAcceptExisting={onAcceptExisting}
                  onRestartPlan={onRestartPlan}
                />
              ) : null}
              {planError ? <PlanErrorCard planError={planError} /> : null}
              {isRunning && !showResumePrompt && !planError ? (
                <PlanRunningIndicator
                  phaseLabel={resolvePhaseLabel()}
                  elapsedSeconds={elapsedSeconds}
                />
              ) : null}
              {!isRunning &&
              !planComplete &&
              !planError &&
              !showResumePrompt &&
              activityEvents.length === 0 ? (
                <div className="flex items-center gap-2">
                  <Loader2 className="h-3.5 w-3.5 animate-spin text-text-dim" />
                  <span className="font-mono text-text-dim text-xs">Initializing agent...</span>
                </div>
              ) : null}
              {cappedEvents.map((event) => (
                <PlanEventRow
                  key={`${event.timestamp}-${event.kind}-${event.content}`}
                  event={event}
                  kindMeta={vocabulary?.planKindMeta ?? {}}
                />
              ))}
              <div ref={activityEndRef} />
            </ScrollContent>
          </ScrollViewport>
        </ScrollArea>
      </CardContent>
      <CardFooter className="border-border border-t p-3">
        {isRunning ? (
          <div className="flex w-full items-center gap-2 font-mono text-text-dim text-xs">
            <Loader2 className="h-3 w-3 animate-spin" />
            <span>Agent is generating plan — input disabled during generation</span>
          </div>
        ) : (
          <div className="flex w-full gap-2">
            <Input
              aria-label="Plan stream input"
              data-testid="plan-stream-input"
              value={userInput}
              placeholder={
                planComplete ? "Describe changes to re-plan..." : "Send message to agent..."
              }
              onChange={(event) => onUserInputChange(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter" && !event.shiftKey) {
                  event.preventDefault();
                  onSendInput();
                }
              }}
              className="h-9 font-mono text-sm"
              disabled={!planComplete && activityEvents.length === 0}
            />
            <Button
              size="sm"
              variant="secondary"
              data-testid="plan-stream-send-button"
              onClick={onSendInput}
              disabled={!planComplete && activityEvents.length === 0}
            >
              {planComplete ? "Re-plan" : "Send"}
            </Button>
          </div>
        )}
      </CardFooter>
    </Card>
  );
}
