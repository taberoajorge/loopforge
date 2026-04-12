import { memo, useEffect, useMemo, useRef } from "react";
import { Loader2 } from "lucide-react";
import { Badge } from "../../../components/ui/badge";
import { ScrollArea, ScrollContent, ScrollViewport } from "../../../components/ui/scroll-area";
import type { AtomizeActivityEvent, AtomizeActivityKind } from "../../../types/wizard";

const KIND_CONFIG: Record<
  AtomizeActivityKind,
  { label: string; variant: "neutral" | "info" | "warning" | "danger" | "success" }
> = {
  planLoaded: { label: "LOAD", variant: "info" },
  templateRender: { label: "TMPL", variant: "neutral" },
  agentStart: { label: "CALL", variant: "warning" },
  agentComplete: { label: "RECV", variant: "success" },
  chunkDetected: { label: "SECT", variant: "info" },
  sectionProcess: { label: "PROC", variant: "warning" },
  storyExtracted: { label: "ATOM", variant: "success" },
  retry: { label: "RTRY", variant: "danger" },
  validation: { label: "VALD", variant: "info" },
  artifactSaved: { label: "SAVE", variant: "success" },
};

const MAX_VISIBLE = 200;

const ActivityRow = memo(function ActivityRow({ event }: { event: AtomizeActivityEvent }) {
  const config = KIND_CONFIG[event.kind];
  const displayTime = event.timestamp.slice(11, 23);
  return (
    <div
      className="flex gap-2 border-b border-border/30 py-1 last:border-0"
      style={{ contentVisibility: "auto", containIntrinsicSize: "auto 28px" }}
    >
      <span className="mt-0.5 shrink-0 text-xs font-mono text-text-dim">{displayTime}</span>
      <Badge
        variant={config.variant}
        className="mt-0.5 h-fit rounded-sm px-1 font-mono text-[10px]"
      >
        {config.label}
      </Badge>
      <span className="break-all text-xs font-mono leading-5 text-text">{event.content}</span>
    </div>
  );
});

type AtomizeStreamPanelProps = {
  events: AtomizeActivityEvent[];
  isRunning: boolean;
  isDone: boolean;
  hasError: boolean;
};

export function AtomizeStreamPanel({ events, isRunning, isDone, hasError }: AtomizeStreamPanelProps) {
  const endRef = useRef<HTMLDivElement>(null);

  const cappedEvents = useMemo(() => {
    if (events.length <= MAX_VISIBLE) return events;
    return events.slice(-MAX_VISIBLE);
  }, [events]);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [cappedEvents.length]);

  return (
    <div className="flex h-full flex-col" role="region" aria-label="Atomization activity stream">
      <ScrollArea className="min-h-0 flex-1">
        <ScrollViewport padding="md" className="h-full">
          <ScrollContent className="space-y-1">
            {isRunning && events.length === 0 ? (
              <div className="flex items-center gap-2 rounded-md border border-border/40 bg-surface/60 px-3 py-2">
                <Loader2 className="h-3.5 w-3.5 shrink-0 animate-spin text-primary" />
                <span className="text-xs font-mono text-text-muted">Starting atomization pipeline...</span>
              </div>
            ) : null}
            {cappedEvents.map((event, index) => (
              <ActivityRow key={`${event.timestamp}-${index}`} event={event} />
            ))}
            {isDone && !hasError ? (
              <div className="flex items-center gap-2 rounded-md border border-primary/30 bg-primary/5 px-3 py-2">
                <Badge variant="success" className="text-[10px]">DONE</Badge>
                <span className="text-xs font-mono text-primary">Pipeline complete</span>
              </div>
            ) : null}
            <div ref={endRef} />
          </ScrollContent>
        </ScrollViewport>
      </ScrollArea>
    </div>
  );
}
