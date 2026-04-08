import type { StateCreator } from "zustand";
import type { WizardConfig } from "../../types/wizard";

export interface WizardConfigSlice {
  config: WizardConfig;
  setConfig: (config: Partial<WizardConfig>) => void;
}

export const DEFAULT_CONFIG: WizardConfig = {
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

export const CONFIG_DEFAULTS = {
  config: DEFAULT_CONFIG,
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
});
