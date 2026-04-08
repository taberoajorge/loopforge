import type { StateCreator } from "zustand";
import type { WizardProjectData } from "../../types/wizard";

const DEFAULT_PROJECT_DATA: WizardProjectData = {
  name: "",
  description: "",
  workingDirectory: "",
  planAgent: "claude",
  planModel: null,
  planEffort: null,
};

export interface WizardStepSlice {
  currentStep: number;
  highestStep: number;
  staleFromStep: number | null;
  projectId: string | null;
  projectData: WizardProjectData;
  setStep: (step: number) => void;
  advanceStep: (step: number) => void;
  markStale: (fromStep: number) => void;
  clearStale: () => void;
  setProjectId: (id: string) => void;
  setProjectData: (data: Partial<WizardProjectData>) => void;
}

export const STEP_DEFAULTS = {
  currentStep: 1,
  highestStep: 1,
  staleFromStep: null as number | null,
  projectId: null as string | null,
  projectData: DEFAULT_PROJECT_DATA,
};

export const createStepSlice: StateCreator<
  WizardStepSlice,
  [],
  [],
  WizardStepSlice
> = (set) => ({
  ...STEP_DEFAULTS,
  setStep: (step) => set({ currentStep: step }),
  advanceStep: (step) => set((state) => ({
    currentStep: step,
    highestStep: Math.max(state.highestStep, step),
    staleFromStep: null,
  })),
  markStale: (fromStep) => set({ staleFromStep: fromStep }),
  clearStale: () => set({ staleFromStep: null }),
  setProjectId: (id) => set({ projectId: id }),
  setProjectData: (data) =>
    set((state) => ({ projectData: { ...state.projectData, ...data } })),
});
