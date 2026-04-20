import type { AgentCapabilities } from "../../../lib/tauri";
import type { WizardConfig } from "../../../types/wizard";
import type { ConfigureValues, ScmProvider } from "../configureFormTypes";
import type { ConfigureErrors } from "../configureValidation";

export type ConfigureState = {
  values: ConfigureValues;
  errors: ConfigureErrors;
  capabilities: AgentCapabilities | null;
};

export type ConfigureAction =
  | { type: "set_values"; values: ConfigureValues }
  | {
      type: "set_value";
      field: keyof ConfigureValues;
      value: ConfigureValues[keyof ConfigureValues];
    }
  | { type: "set_errors"; errors: ConfigureErrors }
  | { type: "set_capabilities"; capabilities: AgentCapabilities | null };

export function toValues(config: WizardConfig): ConfigureValues {
  return {
    executeAgent: config.executeAgent,
    executeModel: config.executeModel ?? null,
    executeEffort: config.executeEffort ?? null,
    fallbackChain: config.fallbackChain,
    gutterThreshold: config.gutterThreshold,
    maxIterations: config.maxIterations,
    cooldownSeconds: config.cooldownSeconds,
    testCommand: config.testCommand,
    maxVerificationRetries: config.maxVerificationRetries,
    scmProvider: config.scmProvider as ScmProvider,
    reviewPollingInterval: config.reviewPollingInterval,
    reviewTimeout: config.reviewTimeout,
    newAgent: "",
  };
}

export function configureFormReducer(
  state: ConfigureState,
  action: ConfigureAction,
): ConfigureState {
  if (action.type === "set_values") {
    return { ...state, values: action.values };
  }
  if (action.type === "set_value") {
    return {
      ...state,
      values: { ...state.values, [action.field]: action.value },
    };
  }
  if (action.type === "set_errors") {
    return { ...state, errors: action.errors };
  }
  return { ...state, capabilities: action.capabilities };
}
