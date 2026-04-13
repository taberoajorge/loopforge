import * as React from "react";
import { Badge, type BadgeProps } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";
import { useDisplayVocabularyStore } from "@/stores/displayVocabularyStore";

export type StepState = "upcoming" | "current" | "complete" | "stale" | "error";

export type StepIndicatorItem = {
  id: string;
  label: React.ReactNode;
  description?: React.ReactNode;
  state: StepState;
  marker?: React.ReactNode;
  disabled?: boolean;
};

const STEP_LABEL_CLASS: Record<StepState, string> = {
  upcoming: "text-text-dim",
  current: "text-text font-medium",
  complete: "text-primary",
  stale: "text-paused",
  error: "text-blocked",
};

export type StepIndicatorProps = React.HTMLAttributes<HTMLDivElement> & {
  steps: StepIndicatorItem[];
  orientation?: "horizontal" | "vertical";
  showConnectors?: boolean;
  onStepSelect?: (step: StepIndicatorItem, stepIndex: number) => void;
};

function resolveMarker(step: StepIndicatorItem, stepIndex: number) {
  if (step.marker) return step.marker;
  if (step.state === "complete") return "✓";
  if (step.state === "stale") return "!";
  if (step.state === "error") return "×";
  return stepIndex + 1;
}

export function StepIndicator({
  className,
  onStepSelect,
  orientation = "horizontal",
  showConnectors = true,
  steps,
  ...props
}: StepIndicatorProps) {
  const vocabulary = useDisplayVocabularyStore((state) => state.vocabulary);
  return (
    <div
      className={cn(
        orientation === "horizontal"
          ? "flex items-center gap-2 overflow-x-auto"
          : "flex flex-col gap-2",
        className,
      )}
      {...props}
    >
      {steps.map((step, stepIndex) => {
        const isClickable = Boolean(onStepSelect) && !step.disabled;
        const variant = (vocabulary?.stepIndicatorVariants[step.state] ?? "neutral") as NonNullable<
          BadgeProps["variant"]
        >;
        const emphasis = (vocabulary?.stepIndicatorEmphasis[step.state] ?? "subtle") as NonNullable<
          BadgeProps["emphasis"]
        >;
        const stepBody = (
          <>
            <Badge variant={variant} emphasis={emphasis} className="min-w-6 justify-center px-1.5">
              {resolveMarker(step, stepIndex)}
            </Badge>
            <span
              className={cn("whitespace-nowrap font-sans text-xs", STEP_LABEL_CLASS[step.state])}
            >
              {step.label}
            </span>
            {step.description ? (
              <span className="whitespace-nowrap font-sans text-[11px] text-text-dim">
                {step.description}
              </span>
            ) : null}
          </>
        );
        return (
          <React.Fragment key={step.id}>
            <div
              className={cn(
                "flex items-center gap-2",
                orientation === "vertical" ? "justify-start" : "",
              )}
            >
              {isClickable ? (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => onStepSelect?.(step, stepIndex)}
                  className="h-auto items-center gap-2 px-0 py-0 hover:bg-transparent"
                >
                  {stepBody}
                </Button>
              ) : (
                <div className="flex h-auto items-center gap-2 px-0 py-0">{stepBody}</div>
              )}
            </div>
            {showConnectors && stepIndex < steps.length - 1 ? (
              <Separator
                tone={step.state === "complete" ? "default" : "muted"}
                orientation={orientation === "horizontal" ? "horizontal" : "vertical"}
                className={cn(orientation === "horizontal" ? "w-6" : "ml-3 h-4")}
              />
            ) : null}
          </React.Fragment>
        );
      })}
    </div>
  );
}
