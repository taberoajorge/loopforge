import { AlertCircle } from "lucide-react";
import { Card, CardContent } from "../../../components/ui/card";

type PlanErrorCardProps = {
  planError: string;
};

export function PlanErrorCard({ planError }: PlanErrorCardProps) {
  return (
    <Card variant="elevated">
      <CardContent className="flex items-start gap-2 p-3">
        <AlertCircle className="mt-0.5 h-4 w-4 shrink-0 text-blocked" />
        <span className="font-mono text-blocked text-xs">{planError}</span>
      </CardContent>
    </Card>
  );
}
