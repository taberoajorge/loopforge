import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useAskStore } from "../../stores/askStore";
import {
  createAgentCapabilities,
  createAskCompletePayload,
  createAskErrorPayload,
  createAskMessage,
  createAskQuestionResult,
  createAskStreamPayload,
} from "../../test/fixtures";
import { emitTauriEvent, mockTauriCommands } from "../../test/mocks";
import { AskTab } from "./AskTab";

function resetAskStore() {
  useAskStore.setState({
    activeProjectId: null,
    messages: [],
    streamingContent: "",
    streamingMessageId: null,
    isAsking: false,
    selectedAgent: "claude",
    selectedModel: null,
    editPrefill: "",
  });
}

describe("AskTab", () => {
  beforeEach(() => {
    resetAskStore();
    Element.prototype.scrollTo ??= () => {};
    vi.spyOn(Element.prototype, "scrollTo").mockImplementation(() => {});
    vi.spyOn(globalThis.crypto, "randomUUID").mockReturnValue(
      "00000000-0000-4000-8000-000000000000",
    );
  });

  it("submits prompts and renders streamed and completed answers", async () => {
    const askQuestion = vi.fn(async () =>
      createAskQuestionResult({
        messageId: "message-002",
        userMessage: createAskMessage({
          id: "message-user-002",
          role: "user",
          content: "What changed?",
          agent: null,
          model: null,
        }),
      }),
    );
    mockTauriCommands({
      ask_history: [],
      ask_question: askQuestion,
      get_known_agents: ["claude", "codex", "cursor", "gemini", "opencode"],
      get_agent_capabilities: createAgentCapabilities({
        supportsModel: false,
        supportsEffort: false,
        models: [],
        efforts: [],
        defaultModel: null,
        defaultEffort: null,
      }),
    });
    const user = userEvent.setup();

    renderAskTab();

    await user.type(
      screen.getByPlaceholderText("Ask about your project..."),
      "What changed?{enter}",
    );

    await waitFor(() =>
      expect(askQuestion).toHaveBeenCalledWith({
        args: {
          projectId: "project-001",
          question: "What changed?",
          agent: "claude",
          model: null,
        },
      }),
    );
    expect(screen.getByText("What changed?")).toBeInTheDocument();
    expect(screen.getByText("Thinking...")).toBeInTheDocument();

    await act(async () => {
      await emitTauriEvent("ask:stream", createAskStreamPayload({ chunk: "Streaming answer" }));
    });

    expect(await screen.findByText("Streaming answer")).toBeInTheDocument();

    await act(async () => {
      await emitTauriEvent(
        "ask:complete",
        createAskCompletePayload({ fullContent: "Final answer", agent: "codex" }),
      );
    });

    expect(await screen.findByText("Final answer")).toBeInTheDocument();
    expect(screen.getByText("codex")).toBeInTheDocument();
    expect(screen.queryByText("Thinking...")).not.toBeInTheDocument();
  });

  it("renders history and error responses from mocked ask events", async () => {
    mockTauriCommands({
      ask_history: [createAskMessage({ role: "assistant", content: "Existing answer" })],
      ask_question: vi.fn(async () =>
        createAskQuestionResult({
          messageId: "message-003",
          userMessage: createAskMessage({
            id: "message-user-003",
            role: "user",
            content: "Need help",
            agent: null,
            model: null,
          }),
        }),
      ),
      get_known_agents: ["claude", "codex", "cursor", "gemini", "opencode"],
      get_agent_capabilities: createAgentCapabilities({
        supportsModel: false,
        supportsEffort: false,
        models: [],
        efforts: [],
        defaultModel: null,
        defaultEffort: null,
      }),
    });
    const user = userEvent.setup();

    renderAskTab();

    expect(await screen.findByText("Existing answer")).toBeInTheDocument();

    await user.type(screen.getByPlaceholderText("Ask about your project..."), "Need help{enter}");
    expect(await screen.findByText("Thinking...")).toBeInTheDocument();

    await act(async () => {
      await emitTauriEvent(
        "ask:error",
        createAskErrorPayload({ messageId: "message-003", error: "Ask failed" }),
      );
    });

    expect(await screen.findByText("Error: Ask failed")).toBeInTheDocument();
    expect(screen.queryByText("Thinking...")).not.toBeInTheDocument();
  });
});

function renderAskTab() {
  return render(<AskTab projectId="project-001" disabled={false} />);
}
