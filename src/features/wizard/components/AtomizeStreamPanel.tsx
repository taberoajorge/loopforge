import { Loader2 } from "lucide-react";
import { memo, useEffect, useMemo, useRef } from "react";
import { Badge } from "../../../components/ui/badge";
import { ScrollArea, ScrollContent, ScrollViewport } from "../../../components/ui/scroll-area";
import { useDisplayVocabularyStore } from "../../../stores/displayVocabularyStore";
import type { AtomizeActivityEvent, AtomizeActivityKind } from "../../../types/wizard";

const ActivityRow = memo(function ActivityRow({
  event,
  kindMeta,
}: {
  event: AtomizeActivityEvent;
  kindMeta: Record<AtomizeActivityKind, { label: string; variant: string }>;
}) {
  const config = kindMeta[event.kind] ?? { label: "WAIT", variant: "neutral" };
  const displayTime = event.timestamp.slice(11, 23);
  return (
    <div
      className="flex gap-2 border-border/30 border-b py-1 last:border-0"
      style={{ contentVisibility: "auto", containIntrinsicSize: "auto 28px" }}
    >
      <span className="mt-0.5 shrink-0 font-mono text-text-dim text-xs">{displayTime}</span>
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

type AtomizeStreamPanelProps = {
  events: AtomizeActivityEvent[];
  isRunning: boolean;
  isDone: boolean;
  hasError: boolean;
};

export function AtomizeStreamPanel({
  events,
  isRunning,
  isDone,
  hasError,
}: AtomizeStreamPanelProps) {
  const endRef = useRef<HTMLDivElement>(null);
  const vocabulary = useDisplayVocabularyStore((state) => state.vocabulary);
  const maxVisible = vocabulary?.maxVisibleActivityEvents ?? 200;
  const kindMeta = (vocabulary?.atomizeActivityKindMeta ?? {}) as Record<
    AtomizeActivityKind,
    { label: string; variant: string }
  >;

  const cappedEvents = useMemo(() => {
    if (events.length <= maxVisible) return events;
    return events.slice(-maxVisible);
  }, [events, maxVisible]);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, []);

  return (
    <section className="flex h-full flex-col" aria-label="Atomization activity stream">
      <ScrollArea className="min-h-0 flex-1">
        <ScrollViewport padding="md" className="h-full">
          <ScrollContent className="space-y-1">
            {isRunning && events.length === 0 ? (
              <div className="flex items-center gap-2 rounded-md border border-border/40 bg-surface/60 px-3 py-2">
                <Loader2 className="h-3.5 w-3.5 shrink-0 animate-spin text-primary" />
                <span className="font-mono text-text-muted text-xs">
                  Starting atomization pipeline...
                </span>
              </div>
            ) : null}
            {cappedEvents.map((event) => (
              <ActivityRow
                key={`${event.timestamp}-${event.kind}-${event.content}`}
                event={event}
                kindMeta={kindMeta}
              />
            ))}
            {isDone && !hasError ? (
              <div className="flex items-center gap-2 rounded-md border border-primary/30 bg-primary/5 px-3 py-2">
                <Badge variant="success" className="text-[10px]">
                  DONE
                </Badge>
                <span className="font-mono text-primary text-xs">Pipeline complete</span>
              </div>
            ) : null}
            <div ref={endRef} />
          </ScrollContent>
        </ScrollViewport>
      </ScrollArea>
    </section>
  );
}
