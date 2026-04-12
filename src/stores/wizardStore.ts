import { create } from "zustand";
import { createStepSlice, STEP_DEFAULTS, type WizardStepSlice } from "./slices/wizard-step";
import { createPlanSlice, PLAN_DEFAULTS, type WizardPlanSlice } from "./slices/wizard-plan";
import { createStoriesSlice, STORIES_DEFAULTS, type WizardStoriesSlice } from "./slices/wizard-stories";
import { createConfigSlice, CONFIG_DEFAULTS, type WizardConfigSlice } from "./slices/wizard-config";

export type { PlanEvent, PlanEventKind, UserStory, WizardProjectData, WizardConfig } from "../types/wizard";

type WizardState = WizardStepSlice & WizardPlanSlice & WizardStoriesSlice & WizardConfigSlice & {
  reset: () => void;
};

export const useWizardStore = create<WizardState>()((...args) => ({
  ...createStepSlice(...args),
  ...createPlanSlice(...args),
  ...createStoriesSlice(...args),
  ...createConfigSlice(...args),
  reset: () => {
    const [set] = args;
    set({
      ...STEP_DEFAULTS,
      ...PLAN_DEFAULTS,
      ...STORIES_DEFAULTS,
      ...CONFIG_DEFAULTS,
    });
  },
}));
