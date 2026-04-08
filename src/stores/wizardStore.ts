import { create } from "zustand";

export type PlanEventKind =
  | "search"
  | "docsLookup"
  | "mcpCall"
  | "planContent"
  | "thinking"
  | "error";

export interface PlanEvent {
  kind: PlanEventKind;
  content: string;
  timestamp: string;
}

export interface UserStory {
  id: string;
  title: string;
  description: string;
  acceptanceCriteria: string[];
  scope: {
    filesToModify: string[];
    filesToCreate: string[];
    filesToAvoid: string[];
  };
  verification: {
    commands: string[];
    assertions: string[];
  };
  commitMessage?: string;
  priority: "critical" | "high" | "medium" | "low";
  estimatedComplexity: "small" | "medium" | "large";
  estimatedMinutes: number;
  dependsOn: string[];
  passes: boolean;
  blocked: boolean;
  attempts: number;
  notes?: string | null;
}

export interface WizardProjectData {
  name: string;
  description: string;
  workingDirectory: string;
  planAgent: string;
  planModel: string | null;
  planEffort: string | null;
}

export interface WizardConfig {
  executeAgent: string;
  executeModel: string | null;
  executeEffort: string | null;
  fallbackChain: string[];
  gutterThreshold: number;
  maxIterations: number;
  cooldownSeconds: number;
  testCommand: string;
  maxVerificationRetries: number;
  scmProvider: "auto" | "github" | "gitlab" | "none";
  reviewPollingInterval: number;
  reviewTimeout: number;
}

interface WizardState {
  currentStep: number;
  highestStep: number;
  staleFromStep: number | null;
  projectId: string | null;
  projectData: WizardProjectData;
  planEvents: PlanEvent[];
  planContent: string;
  planComplete: boolean;
  planRunning: boolean;
  stories: UserStory[];
  config: WizardConfig;
  setStep: (step: number) => void;
  advanceStep: (step: number) => void;
  markStale: (fromStep: number) => void;
  clearStale: () => void;
  setProjectId: (id: string) => void;
  setProjectData: (data: Partial<WizardProjectData>) => void;
  appendPlanEvent: (event: PlanEvent) => void;
  appendPlanEvents: (events: PlanEvent[]) => void;
  appendPlanContent: (text: string) => void;
  appendPlanContentDelta: (delta: string) => void;
  setPlanComplete: (complete: boolean) => void;
  setPlanRunning: (running: boolean) => void;
  setStories: (stories: UserStory[]) => void;
  updateStory: (id: string, patch: Partial<UserStory>) => void;
  reorderStories: (fromIndex: number, toIndex: number) => void;
  addStory: (story: UserStory) => void;
  removeStory: (id: string) => void;
  setConfig: (config: Partial<WizardConfig>) => void;
  reset: () => void;
}

const DEFAULT_PROJECT_DATA: WizardProjectData = {
  name: "",
  description: "",
  workingDirectory: "",
  planAgent: "claude",
  planModel: null,
  planEffort: null,
};

const DEFAULT_CONFIG: WizardConfig = {
  executeAgent: "cursor",
  executeModel: null,
  executeEffort: null,
  fallbackChain: ["claude"],
  gutterThreshold: 3,
  maxIterations: 50,
  cooldownSeconds: 5,
  testCommand: "",
  maxVerificationRetries: 3,
  scmProvider: "auto",
  reviewPollingInterval: 60,
  reviewTimeout: 600,
};

export const useWizardStore = create<WizardState>()((set) => ({
  currentStep: 1,
  highestStep: 1,
  staleFromStep: null,
  projectId: null,
  projectData: DEFAULT_PROJECT_DATA,
  planEvents: [],
  planContent: "",
  planComplete: false,
  planRunning: false,
  stories: [],
  config: DEFAULT_CONFIG,
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
  setStories: (stories) => set({ stories }),
  updateStory: (id, patch) =>
    set((state) => ({
      stories: state.stories.map((story) =>
        story.id === id ? { ...story, ...patch } : story,
      ),
    })),
  reorderStories: (fromIndex, toIndex) =>
    set((state) => {
      const reordered = [...state.stories];
      const [moved] = reordered.splice(fromIndex, 1);
      reordered.splice(toIndex, 0, moved);
      return { stories: reordered };
    }),
  addStory: (story) =>
    set((state) => ({ stories: [...state.stories, story] })),
  removeStory: (id) =>
    set((state) => ({
      stories: state.stories.filter((story) => story.id !== id),
    })),
  setConfig: (config) =>
    set((state) => ({ config: { ...state.config, ...config } })),
  reset: () =>
    set({
      currentStep: 1,
      highestStep: 1,
      staleFromStep: null,
      projectId: null,
      projectData: DEFAULT_PROJECT_DATA,
      planEvents: [],
      planContent: "",
      planComplete: false,
      planRunning: false,
      stories: [],
      config: DEFAULT_CONFIG,
    }),
}));
