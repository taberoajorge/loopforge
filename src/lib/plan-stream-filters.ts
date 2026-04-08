import type { PlanEvent } from "../types/wizard";

const EXACT_NOISE = new Set([
  "exec", "codex", "--------", "---", "WAIT", "reason", "effort", "found",
]);

const PREFIX_NOISE = [
  "OpenAI Codex", "workdir:", "model:", "provider:", "approval:", "sandbox:",
  "reasoning effort:", "reasoning summaries:", "session id:",
  "user You are a senior software architect.",
  "error: unexpected argument", "tip: to pass", "Usage: codex",
  "For more information", "Usage:", "tip:",
  "Reading additional input from stdin", "Warning: no stdin data received",
  "If piping from a slow command",
];

export function isNoisePlanLine(content: string): boolean {
  const trimmed = content.trim();
  if (!trimmed || EXACT_NOISE.has(trimmed)) return true;
  if (PREFIX_NOISE.some((prefix) => trimmed.startsWith(prefix))) return true;
  return false;
}

export function normalizePlanLine(content: string): string {
  const trimmed = content.trim();
  if (trimmed.startsWith("codex ")) return trimmed.slice(6).trim();
  if (trimmed.startsWith("user ")) return trimmed.slice(5).trim();
  return trimmed;
}

export function shouldRenderPlanEvent(event: PlanEvent): boolean {
  return !isNoisePlanLine(event.content);
}
