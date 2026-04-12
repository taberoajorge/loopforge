import { useRef, useState } from "react";
import { Outlet, useLocation, useNavigate, useParams } from "react-router";
import { type StepState } from "../../components/StepIndicator";
import {
  AlertDialog, AlertDialogAction, AlertDialogCancel,
  AlertDialogContent, AlertDialogDescription,
  AlertDialogFooter, AlertDialogHeader, AlertDialogTitle,
} from "../../components/ui/alert-dialog";
import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";
import { Separator } from "../../components/ui/separator";
import { useWizardStore } from "../../stores/wizardStore";
import { stopPlan, saveWizardDraft } from "../../lib/tauri";
import { useWizardHydration } from "../../hooks/useWizardHydration";
import { WizardExitDialog } from "./components/WizardExitDialog";
import { WizardStepRail } from "./components/WizardStepRail";

const WIZARD_STEPS = [
  { number: 1, label: "Describe", slug: "describe" },
  { number: 2, label: "Plan", slug: "plan" },
  { number: 3, label: "Atomize", slug: "atomize" },
  { number: 4, label: "Configure", slug: "configure" },
  { number: 5, label: "Launch", slug: "launch" },
];

function resolveStep(pathname: string): number {
  if (pathname.startsWith("/new/launch")) return 5;
  if (pathname.startsWith("/new/configure")) return 4;
  if (pathname.startsWith("/new/atomize")) return 3;
  if (pathname.startsWith("/new/plan")) return 2;
  return 1;
}

export function WizardLayout() {
  const location = useLocation();
  const navigate = useNavigate();
  const params = useParams<{ id?: string }>();
  const projectId = useWizardStore((state) => state.projectId);
  const projectName = useWizardStore((state) => state.projectData.name);
  const staleFromStep = useWizardStore((state) => state.staleFromStep);
  const highestStep = useWizardStore((state) => state.highestStep);
  const planRunning = useWizardStore((state) => state.planRunning);
  const currentStep = resolveStep(location.pathname);
  const [showExitConfirm, setShowExitConfirm] = useState(false);
  const [showLeavePlanConfirm, setShowLeavePlanConfirm] = useState(false);
  const pendingNavRef = useRef<(() => void) | null>(null);

  const { hydrating } = useWizardHydration(params.id, projectId, projectName);

  async function handleGoHome() {
    if (projectId) {
      if (currentStep === 2) {
        await stopPlan(projectId).catch(() => {});
      }
      const stepSlug = WIZARD_STEPS.find((step) => step.number === currentStep)?.slug ?? "describe";
      await saveWizardDraft(projectId, stepSlug).catch(() => {});
    }
    navigate("/");
  }

  function executeNav(step: typeof WIZARD_STEPS[number]) {
    const path = step.number === 1
      ? "/new/describe"
      : `/new/${step.slug}/${projectId}`;
    useWizardStore.getState().setStep(step.number);
    navigate(path);
  }

  function handleStepClick(step: typeof WIZARD_STEPS[number]) {
    if (step.number === currentStep) return;
    if (step.number > highestStep) return;
    if (!projectId && step.number > 1) return;

    if (currentStep === 2 && planRunning && step.number !== 2) {
      pendingNavRef.current = () => executeNav(step);
      setShowLeavePlanConfirm(true);
      return;
    }

    executeNav(step);
  }

  const activeStepLabel = WIZARD_STEPS.find((step) => step.number === currentStep)?.label ?? "Describe";

  const stepItems = WIZARD_STEPS.map((step) => {
    const isCompleted = step.number < currentStep;
    const isStale = staleFromStep !== null && step.number >= staleFromStep && isCompleted;
    const state: StepState = step.number === currentStep
      ? "current"
      : isStale
        ? "stale"
        : isCompleted
          ? "complete"
          : "upcoming";
    return {
      id: step.slug,
      label: step.label,
      state,
      disabled: step.number === currentStep || step.number > highestStep,
    };
  });

  return (
    <div className="flex h-full overflow-hidden">
      <WizardStepRail
        projectName={projectName}
        currentStep={currentStep}
        steps={stepItems}
        onStepSelect={(stepIndex) => {
          const selectedStep = WIZARD_STEPS[stepIndex];
          if (!selectedStep) return;
          handleStepClick(selectedStep);
        }}
      />
      <div className="flex min-w-0 flex-1 flex-col">
        <div className="border-b border-border bg-surface px-4 py-3 lg:hidden">
          <div className="flex items-center gap-2">
            <Button variant="ghost" size="sm" className="h-7 px-2 text-xs" onClick={() => setShowExitConfirm(true)}>
              Home
            </Button>
            <Separator orientation="vertical" className="h-5" />
            <Badge variant="info">Step {currentStep}</Badge>
            <span className="truncate text-xs font-sans text-text-muted">
              {activeStepLabel}
            </span>
          </div>
        </div>
        <div className="flex-1 min-h-0">
          {hydrating ? (
            <div className="flex h-full items-center justify-center">
              <p className="text-sm font-mono text-text-dim">Loading draft...</p>
            </div>
          ) : (
            <Outlet />
          )}
        </div>
      </div>
      <WizardExitDialog
        open={showExitConfirm}
        onOpenChange={setShowExitConfirm}
        onConfirm={() => {
          setShowExitConfirm(false);
          void handleGoHome();
        }}
      />
      <AlertDialog open={showLeavePlanConfirm} onOpenChange={setShowLeavePlanConfirm}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Plan is still running</AlertDialogTitle>
            <AlertDialogDescription>
              The plan agent is still generating output. It will continue running in the background.
              You can return to this step to check its progress.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel asChild>
              <Button variant="secondary">Stay</Button>
            </AlertDialogCancel>
            <AlertDialogAction asChild onClick={() => {
              setShowLeavePlanConfirm(false);
              pendingNavRef.current?.();
              pendingNavRef.current = null;
            }}>
              <Button variant="primary">Leave Anyway</Button>
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
