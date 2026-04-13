import type { StateCreator } from "zustand";
import type { PlanEvent } from "../../types/wizard";

export interface WizardPlanSlice {
  planEvents: PlanEvent[];
  planContent: string;
  planComplete: boolean;
  planRunning: boolean;
  appendPlanEvent: (event: PlanEvent) => void;
  appendPlanEvents: (events: PlanEvent[]) => void;
  setPlanContent: (content: string) => void;
  setPlanComplete: (complete: boolean) => void;
  setPlanRunning: (running: boolean) => void;
}

export const PLAN_DEFAULTS = {
  planEvents: [] as PlanEvent[],
  planContent: "",
  planComplete: false,
  planRunning: false,
};

export const createPlanSlice: StateCreator<WizardPlanSlice, [], [], WizardPlanSlice> = (set) => ({
  ...PLAN_DEFAULTS,
  appendPlanEvent: (event) => set((state) => ({ planEvents: [...state.planEvents, event] })),
  appendPlanEvents: (events) =>
    set((state) => {
      if (events.length === 0) return state;
      return { planEvents: [...state.planEvents, ...events] };
    }),
  setPlanContent: (content) => set({ planContent: content }),
  setPlanComplete: (complete) => set({ planComplete: complete }),
  setPlanRunning: (running) => set({ planRunning: running }),
});
