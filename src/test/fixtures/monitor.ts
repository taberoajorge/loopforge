import type {
  AgentOutputPayload,
  AskCompletePayload,
  AskErrorPayload,
  AskMessage,
  AskStreamPayload,
  HeartbeatPayload,
  IterationRow,
  PromptPayload,
  StorySkippedPayload,
  VerificationPayload,
} from "../../lib/tauri";
import type { ProjectScopedEventPayload } from "../mocks";
import { mergeFixture, type DeepPartial } from "./shared";

export function createProjectScopedEventPayload(
  overrides?: DeepPartial<ProjectScopedEventPayload>,
): ProjectScopedEventPayload {
  return mergeFixture<ProjectScopedEventPayload>({
    projectId: "project-001",
    sessionId: "session-001",
    storyId: "S-005",
  }, overrides);
}

export function createIterationRow(overrides?: DeepPartial<IterationRow>): IterationRow {
  return mergeFixture<IterationRow>({
    storyId: "S-005",
    startedAt: "2026-04-09T10:30:00.000Z",
    durationSecs: 45,
    result: "passed",
    agentUsed: "codex",
  }, overrides);
}

export function createAskMessage(overrides?: DeepPartial<AskMessage>): AskMessage {
  return mergeFixture<AskMessage>({
    id: "message-001",
    conversationId: "conversation-001",
    role: "assistant",
    content: "All checks passed.",
    agent: "codex",
    model: "gpt-5.4",
    createdAt: "2026-04-09T10:35:00.000Z",
  }, overrides);
}

export function createAskStreamPayload(
  overrides?: DeepPartial<AskStreamPayload>,
): AskStreamPayload {
  return mergeFixture<AskStreamPayload>({
    projectId: "project-001",
    messageId: "message-001",
    chunk: "Streaming answer",
  }, overrides);
}

export function createAskCompletePayload(
  overrides?: DeepPartial<AskCompletePayload>,
): AskCompletePayload {
  return mergeFixture<AskCompletePayload>({
    projectId: "project-001",
    messageId: "message-001",
    fullContent: "Final answer",
    agent: "codex",
    model: "gpt-5.4",
  }, overrides);
}

export function createAskErrorPayload(
  overrides?: DeepPartial<AskErrorPayload>,
): AskErrorPayload {
  return mergeFixture<AskErrorPayload>({
    projectId: "project-001",
    messageId: "message-001",
    error: "Ask failed",
  }, overrides);
}

export function createHeartbeatPayload(
  overrides?: DeepPartial<HeartbeatPayload>,
): HeartbeatPayload {
  return mergeFixture<HeartbeatPayload>({
    projectId: "project-001",
    sessionId: "session-001",
    elapsedSecs: 90,
    totalSecs: 300,
    context: "cooldown_wait",
  }, overrides);
}

export function createVerificationPayload(
  overrides?: DeepPartial<VerificationPayload>,
): VerificationPayload {
  return mergeFixture<VerificationPayload>({
    projectId: "project-001",
    sessionId: "session-001",
    storyId: "S-005",
    attempt: 1,
    maxAttempts: 3,
    errorCount: 0,
    circuitBreaker: false,
  }, overrides);
}

export function createPromptPayload(overrides?: DeepPartial<PromptPayload>): PromptPayload {
  return mergeFixture<PromptPayload>({
    projectId: "project-001",
    sessionId: "session-001",
    storyId: "S-005",
    sizeBytes: 512,
    hash: 123456,
    truncated: false,
  }, overrides);
}

export function createStorySkippedPayload(
  overrides?: DeepPartial<StorySkippedPayload>,
): StorySkippedPayload {
  return mergeFixture<StorySkippedPayload>({
    projectId: "project-001",
    sessionId: "session-001",
    storyId: "S-005",
    reason: "Already passed",
  }, overrides);
}

export function createAgentOutputPayload(
  overrides?: DeepPartial<AgentOutputPayload>,
): AgentOutputPayload {
  return mergeFixture<AgentOutputPayload>({
    projectId: "project-001",
    sessionId: "session-001",
    line: "Running tests",
    stream: "stdout",
  }, overrides);
}
