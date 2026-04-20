import { SectionHeader } from "@/components/SectionHeader";
import { Card, CardContent } from "@/components/ui/card";

type SystemStatusCardProps = {
  activeCount: number;
  totalCount: number;
};

export function SystemStatusCard({ activeCount, totalCount }: SystemStatusCardProps) {
  return (
    <Card className="h-fit">
      <CardContent className="space-y-2 p-3">
        <SectionHeader
          title={<span className="ui-type-label">SYSTEM STATUS</span>}
          compact
          className="space-y-1"
        />
        <div className="space-y-1.5">
          <div className="flex items-center justify-between rounded-md border border-border/60 bg-elevated px-2 py-1.5">
            <span className="ui-type-label font-sans text-text-muted uppercase tracking-[0.14em]">
              Compute Load
            </span>
            <span className="ui-type-body font-mono text-running">{activeCount} active</span>
          </div>
          <div className="flex items-center justify-between rounded-md border border-border/60 bg-elevated px-2 py-1.5">
            <span className="ui-type-label font-sans text-text-muted uppercase tracking-[0.14em]">
              Total Loops
            </span>
            <span className="ui-type-body font-mono text-text">{totalCount}</span>
          </div>
          <div className="flex items-center justify-between rounded-md border border-border/60 bg-elevated px-2 py-1.5">
            <span className="ui-type-label font-sans text-text-muted uppercase tracking-[0.14em]">
              Engine
            </span>
            <span className="ui-type-body font-mono text-success">Ready</span>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
