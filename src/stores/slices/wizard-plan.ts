import type { StateCreator } from "zustand";
import type { PlanEvent } from "../../types/wizard";

export interface WizardPlanSlice {
  planEvents: PlanEvent[];
  planContent: string;
  planComplete: boolean;
  planRunning: boolean;
  appendPlanEvent: (event: PlanEvent) => void;
  appendPlanEvents: (events: PlanEvent[]) => void;
  appendPlanContent: (text: string) => void;
  appendPlanContentDelta: (delta: string) => void;
  setPlanComplete: (complete: boolean) => void;
  setPlanRunning: (running: boolean) => void;
}

export const PLAN_DEFAULTS = {
  planEvents: [] as PlanEvent[],
  planContent: "",
  planComplete: false,
  planRunning: false,
};

export const createPlanSlice: StateCreator<
  WizardPlanSlice,
  [],
  [],
  WizardPlanSlice
> = (set) => ({
  ...PLAN_DEFAULTS,
  appendPlanEvent: (event) =>
    set((state) => ({ planEvents: [...state.planEvents, event] })),
  appendPlanEvents: (events) =>
    set((state) => {
      if (events.length === 0) return state;
      return { planEvents: [...state.planEvents, ...events] };
    }),
  appendPlanContent: (text) =>
    set((state) => ({
      planContent: state.planContent
        ? state.planContent + "\n" + text
        : text,
    })),
  appendPlanContentDelta: (delta) =>
    set((state) => {
      if (!delta) return state;
      return {
        planContent: state.planContent
          ? state.planContent + "\n" + delta
          : delta,
      };
    }),
  setPlanComplete: (complete) => set({ planComplete: complete }),
  setPlanRunning: (running) => set({ planRunning: running }),
});
