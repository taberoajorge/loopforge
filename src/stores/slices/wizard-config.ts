import type { StateCreator } from "zustand";
import type { WizardConfig } from "../../types/wizard";

export interface WizardConfigSlice {
  config: WizardConfig;
  configLoaded: boolean;
  setConfig: (config: Partial<WizardConfig>) => void;
  setFullConfig: (config: WizardConfig) => void;
  setConfigLoaded: (loaded: boolean) => void;
}

const PLACEHOLDER_CONFIG: WizardConfig = {
  executeAgent: "",
  executeModel: null,
  executeEffort: null,
  fallbackChain: [],
  gutterThreshold: 0,
  maxIterations: 0,
  cooldownSeconds: 0,
  testCommand: "",
  maxVerificationRetries: 0,
  scmProvider: "auto",
  reviewPollingInterval: 0,
  reviewTimeout: 0,
};

export const CONFIG_DEFAULTS = {
  config: PLACEHOLDER_CONFIG,
  configLoaded: false,
};

export const createConfigSlice: StateCreator<
  WizardConfigSlice,
  [],
  [],
  WizardConfigSlice
> = (set) => ({
  ...CONFIG_DEFAULTS,
  setConfig: (config) =>
    set((state) => ({ config: { ...state.config, ...config } })),
  setFullConfig: (config) => set({ config, configLoaded: true }),
  setConfigLoaded: (loaded) => set({ configLoaded: loaded }),
});
