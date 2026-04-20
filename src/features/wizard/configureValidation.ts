export type ConfigureField =
  | "executeAgent"
  | "gutterThreshold"
  | "maxIterations"
  | "cooldownSeconds"
  | "maxVerificationRetries"
  | "reviewPollingInterval"
  | "reviewTimeout";

export type ConfigureErrors = Partial<Record<ConfigureField, string>>;
