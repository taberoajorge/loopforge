import { Loader2 } from "lucide-react";

type PlanRunningIndicatorProps = {
  phaseLabel: string;
  elapsedSeconds: number;
};

export function PlanRunningIndicator({ phaseLabel, elapsedSeconds }: PlanRunningIndicatorProps) {
  return (
    <div className="flex items-center gap-2 rounded-md border border-border/40 bg-surface/60 px-3 py-2">
      <Loader2 className="h-3.5 w-3.5 shrink-0 animate-spin text-primary" />
      <span className="font-mono text-text-muted text-xs">{phaseLabel}</span>
      <span className="ml-auto font-mono text-[10px] text-text-dim tabular-nums">
        {elapsedSeconds}s
      </span>
    </div>
  );
}
