import { Button } from "../../../components/ui/button";
import { Card, CardContent } from "../../../components/ui/card";

type PlanResumePromptProps = {
  onAcceptExisting: () => void;
  onRestartPlan: () => void;
};

export function PlanResumePrompt({ onAcceptExisting, onRestartPlan }: PlanResumePromptProps) {
  return (
    <Card variant="elevated">
      <CardContent className="space-y-3 p-4">
        <p className="text-sm text-text">
          An existing plan was found. Continue with it or start fresh?
        </p>
        <div className="flex gap-2">
          <Button
            size="sm"
            data-testid="plan-stream-use-existing-button"
            onClick={onAcceptExisting}
          >
            Use Existing Plan
          </Button>
          <Button
            size="sm"
            variant="secondary"
            data-testid="plan-stream-start-fresh-button"
            onClick={onRestartPlan}
          >
            Start Fresh
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
