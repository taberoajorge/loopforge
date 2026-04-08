import { memo, useEffect, useMemo, useRef, useState, type RefObject } from "react";
import { AlertCircle, Loader2 } from "lucide-react";
import { Badge } from "../../../components/ui/badge";
import { Button } from "../../../components/ui/button";
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from "../../../components/ui/card";
import { Input } from "../../../components/ui/input";
import { ScrollArea, ScrollContent, ScrollViewport } from "../../../components/ui/scroll-area";
import { Separator } from "../../../components/ui/separator";
import type { PlanEvent, PlanEventKind } from "../../../types/wizard";
export { shouldRenderPlanEvent } from "../../../lib/plan-stream-filters";

const MAX_VISIBLE_EVENTS = 200;

const KIND_CONFIG: Record<
  PlanEventKind,
  { label: string; variant: "neutral" | "info" | "warning" | "danger" | "success" }
> = {
  search: { label: "SRCH", variant: "info" },
  docsLookup: { label: "DOCS", variant: "warning" },
  mcpCall: { label: "TOOL", variant: "neutral" },
  thinking: { label: "WAIT", variant: "neutral" },
  error: { label: "ERR", variant: "danger" },
  planContent: { label: "PLAN", variant: "success" },
};

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

const PlanEventRow = memo(function PlanEventRow({ event }: { event: PlanEvent }) {
  const config = KIND_CONFIG[event.kind];
  return (
    <div
      className="flex gap-2 border-b border-border/30 py-1 last:border-0"
      style={{ contentVisibility: "auto", containIntrinsicSize: "auto 28px" }}
    >
      <span className="mt-0.5 shrink-0 text-xs font-mono text-text-dim">
        {event.timestamp}
      </span>
      <Badge
        variant={config.variant}
        className="mt-0.5 h-fit rounded-sm px-1 font-mono text-[10px]"
      >
        {config.label}
      </Badge>
      <span className="break-all text-xs font-mono leading-5 text-text">
        {event.content}
      </span>
    </div>
  );
});

function useElapsedSeconds(active: boolean) {
  const [elapsed, setElapsed] = useState(0);
  const startRef = useRef(Date.now());
  useEffect(() => {
    if (!active) {
      setElapsed(0);
      startRef.current = Date.now();
      return;
    }
    startRef.current = Date.now();
    const timer = setInterval(
      () => setElapsed(Math.floor((Date.now() - startRef.current) / 1000)),
      1000,
    );
    return () => clearInterval(timer);
  }, [active]);
  return elapsed;
}

export function PlanStreamPanel(props: PlanStreamPanelProps) {
  const {
    activityEvents, allEvents, planStarted, planComplete,
    showResumePrompt, planError, userInput, onUserInputChange,
    onSendInput, onAcceptExisting, onRestartPlan, activityEndRef,
  } = props;

  const elapsed = useElapsedSeconds(planStarted && !planComplete);
  const isRunning = planStarted && !planComplete;

  const { hasSearch, hasDocsLookup, hasPlanContent } = useMemo(() => {
    let search = false;
    let docs = false;
    let plan = false;
    for (const evt of allEvents) {
      if (evt.kind === "search") search = true;
      else if (evt.kind === "docsLookup") docs = true;
      else if (evt.kind === "planContent") plan = true;
      if (search && docs && plan) break;
    }
    return { hasSearch: search, hasDocsLookup: docs, hasPlanContent: plan };
  }, [allEvents]);

  const cappedEvents = useMemo(() => {
    if (activityEvents.length <= MAX_VISIBLE_EVENTS) return activityEvents;
    return activityEvents.slice(-MAX_VISIBLE_EVENTS);
  }, [activityEvents]);
  const lastEventTimeRef = useRef(Date.now());
  const [secondsSinceLastEvent, setSecondsSinceLastEvent] = useState(0);

  useEffect(() => {
    lastEventTimeRef.current = Date.now();
  }, [allEvents.length]);

  useEffect(() => {
    if (!isRunning) {
      setSecondsSinceLastEvent(0);
      return;
    }
    const timer = setInterval(
      () => setSecondsSinceLastEvent(
        Math.floor((Date.now() - lastEventTimeRef.current) / 1000),
      ),
      1000,
    );
    return () => clearInterval(timer);
  }, [isRunning]);

  const STALL_THRESHOLD = 120;
  const isStalled = isRunning && secondsSinceLastEvent > STALL_THRESHOLD;

  function resolvePhaseLabel(): string {
    if (isStalled) return `No output for ${secondsSinceLastEvent}s — agent may be stalled`;
    if (!planStarted) return "Initializing agent...";
    if (hasPlanContent) return "Writing plan...";
    if (hasDocsLookup) return "Analyzing documentation...";
    if (hasSearch) return "Researching codebase...";
    if (activityEvents.length > 0) return "Agent working...";
    if (elapsed < 20) return "Connecting to agent...";
    return "Waiting for agent response...";
  }

  function resolveStatusBadge() {
    if (planError) return <Badge variant="danger">Error</Badge>;
    if (planStarted && !planComplete) {
      return <Badge variant="warning" className="animate-pulse">Running</Badge>;
    }
    if (planComplete) return <Badge variant="success">Complete</Badge>;
    return null;
  }

  return (
    <Card className="flex h-full min-h-0 flex-col overflow-hidden rounded-none border-0 border-r">
      <CardHeader className="gap-3 p-3">
        <div className="flex items-center gap-2">
          <CardTitle className="text-xs font-mono uppercase tracking-widest text-text-muted">
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
                <Card variant="elevated">
                  <CardContent className="space-y-3 p-4">
                    <p className="text-sm text-text">
                      An existing plan was found. Continue with it or start fresh?
                    </p>
                    <div className="flex gap-2">
                      <Button size="sm" onClick={onAcceptExisting}>
                        Use Existing Plan
                      </Button>
                      <Button size="sm" variant="secondary" onClick={onRestartPlan}>
                        Start Fresh
                      </Button>
                    </div>
                  </CardContent>
                </Card>
              ) : null}
              {planError ? (
                <Card variant="elevated">
                  <CardContent className="flex items-start gap-2 p-3">
                    <AlertCircle className="mt-0.5 h-4 w-4 shrink-0 text-blocked" />
                    <span className="text-xs font-mono text-blocked">
                      {planError}
                    </span>
                  </CardContent>
                </Card>
              ) : null}
              {isRunning && !showResumePrompt && !planError ? (
                <div className="flex items-center gap-2 rounded-md border border-border/40 bg-surface/60 px-3 py-2">
                  <Loader2 className="h-3.5 w-3.5 shrink-0 animate-spin text-primary" />
                  <span className="text-xs font-mono text-text-muted">
                    {resolvePhaseLabel()}
                  </span>
                  <span className="ml-auto text-[10px] font-mono text-text-dim tabular-nums">
                    {elapsed}s
                  </span>
                </div>
              ) : null}
              {!isRunning && !planComplete && !planError && !showResumePrompt && activityEvents.length === 0 ? (
                <div className="flex items-center gap-2">
                  <Loader2 className="h-3.5 w-3.5 animate-spin text-text-dim" />
                  <span className="text-xs font-mono text-text-dim">
                    Initializing agent...
                  </span>
                </div>
              ) : null}
              {cappedEvents.map((event, index) => (
                <PlanEventRow key={`${event.timestamp}-${index}`} event={event} />
              ))}
              <div ref={activityEndRef} />
            </ScrollContent>
          </ScrollViewport>
        </ScrollArea>
      </CardContent>
      <CardFooter className="border-t border-border p-3">
        {isRunning ? (
          <div className="flex w-full items-center gap-2 text-xs font-mono text-text-dim">
            <Loader2 className="h-3 w-3 animate-spin" />
            <span>Agent is generating plan — input disabled during generation</span>
          </div>
        ) : (
          <div className="flex w-full gap-2">
            <Input
              value={userInput}
              placeholder={planComplete ? "Describe changes to re-plan..." : "Send message to agent..."}
              onChange={(event) => onUserInputChange(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter" && !event.shiftKey) {
                  event.preventDefault();
                  onSendInput();
                }
              }}
              className="h-9 text-sm font-mono"
              disabled={!planComplete && activityEvents.length === 0}
            />
            <Button
              size="sm" variant="secondary"
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
