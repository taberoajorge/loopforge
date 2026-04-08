import { Badge } from "../../../components/ui/badge";
import { Separator } from "../../../components/ui/separator";
import { StepIndicator, type StepIndicatorItem } from "../../../components/StepIndicator";

type WizardStepRailProps = {
  projectName: string;
  currentStep: number;
  steps: StepIndicatorItem[];
  onStepSelect: (stepIndex: number) => void;
};

export function WizardStepRail({
  projectName,
  currentStep,
  steps,
  onStepSelect,
}: WizardStepRailProps) {
  return (
    <aside className="hidden w-56 shrink-0 border-r border-border bg-surface/40 lg:block">
      <div className="flex h-full flex-col">
        <div className="space-y-3 p-4">
          <Badge variant="info">Setup Flow</Badge>
          <p className="truncate text-sm font-sans font-medium text-text">
            {projectName || "New project"}
          </p>
          <p className="text-[11px] font-sans uppercase tracking-widest text-text-dim">
            Step {currentStep} of {steps.length}
          </p>
        </div>
        <Separator tone="muted" />
        <div className="flex-1 overflow-y-auto p-4">
          <StepIndicator
            steps={steps}
            orientation="vertical"
            className="items-start"
            onStepSelect={(selectedStepItem, stepIndex) => {
              void selectedStepItem;
              onStepSelect(stepIndex);
            }}
          />
        </div>
      </div>
    </aside>
  );
}
