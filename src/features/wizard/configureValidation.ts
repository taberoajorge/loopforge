export type ConfigureField =
  | "executeAgent"
  | "gutterThreshold"
  | "maxIterations"
  | "cooldownSeconds"
  | "maxVerificationRetries"
  | "reviewPollingInterval"
  | "reviewTimeout";

export type ConfigureErrors = Partial<Record<ConfigureField, string>>;

const LIMITS = {
  gutterThreshold: [1, 20, "Gutter threshold"],
  maxIterations: [1, 500, "Max iterations"],
  cooldownSeconds: [0, 300, "Cooldown"],
  maxVerificationRetries: [1, 10, "Verification retries"],
  reviewPollingInterval: [10, 600, "Poll interval"],
  reviewTimeout: [60, 3600, "Timeout"],
} as const;

export function validateConfig(values: {
  executeAgent: string;
  gutterThreshold: number;
  maxIterations: number;
  cooldownSeconds: number;
  maxVerificationRetries: number;
  reviewPollingInterval: number;
  reviewTimeout: number;
}): ConfigureErrors {
  const nextErrors: ConfigureErrors = {};
  if (!values.executeAgent.trim()) nextErrors.executeAgent = "Execute agent is required";
  for (const [field, value] of [
    ["gutterThreshold", values.gutterThreshold],
    ["maxIterations", values.maxIterations],
    ["cooldownSeconds", values.cooldownSeconds],
    ["maxVerificationRetries", values.maxVerificationRetries],
    ["reviewPollingInterval", values.reviewPollingInterval],
    ["reviewTimeout", values.reviewTimeout],
  ] as const) {
    const [min, max, label] = LIMITS[field];
    if (!Number.isFinite(value) || value < min || value > max) {
      nextErrors[field] = `${label} must be between ${min} and ${max}`;
    }
  }
  return nextErrors;
}
