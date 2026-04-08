import { useCallback, useEffect, useRef } from "react";
import { MessageSquare } from "lucide-react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { EmptyState } from "../../components/EmptyState";
import { MARKDOWN_COMPONENTS } from "../../components/markdownComponents";
import { Badge } from "../../components/ui/badge";
import {
  askHistory,
  askQuestion,
  copyAskMessage,
  onAskComplete,
  onAskError,
  onAskStream,
  retryAsk,
  stopAsk,
  truncateAskFrom,
  type AskMessage,
} from "../../lib/tauri";
import { useAskStore } from "../../stores/askStore";
import { AskInput } from "./components/AskInput";
import { AskMessageBubble } from "./components/AskMessage";
import { useShallow } from "zustand/react/shallow";

type AskTabProps = {
  projectId: string;
  disabled: boolean;
};

export function AskTab({ projectId, disabled }: AskTabProps) {
  const {
    messages, streamingContent, isAsking, selectedAgent,
    selectedModel, editPrefill,
  } = useAskStore(useShallow((state) => ({
    messages: state.messages,
    streamingContent: state.streamingContent,
    isAsking: state.isAsking,
    selectedAgent: state.selectedAgent,
    selectedModel: state.selectedModel,
    editPrefill: state.editPrefill,
  })));

  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    useAskStore.getState().switchProject(projectId);
    askHistory(projectId)
      .then((history) => useAskStore.getState().setMessages(history))
      .catch(() => {});
  }, [projectId]);

  useEffect(() => {
    const unsubStream = onAskStream((payload) => {
      if (payload.projectId !== projectId) return;
      useAskStore.getState().appendStreamChunk(payload.chunk);
    });
    const unsubComplete = onAskComplete((payload) => {
      if (payload.projectId !== projectId) return;
      const message: AskMessage = {
        id: payload.messageId,
        conversationId: "",
        role: "assistant",
        content: payload.fullContent,
        agent: payload.agent,
        model: payload.model,
        createdAt: new Date().toISOString(),
      };
      useAskStore.getState().finalize(message);
    });
    const unsubError = onAskError((payload) => {
      if (payload.projectId !== projectId) return;
      const message: AskMessage = {
        id: payload.messageId,
        conversationId: "",
        role: "assistant",
        content: `Error: ${payload.error}`,
        agent: null,
        model: null,
        createdAt: new Date().toISOString(),
      };
      useAskStore.getState().finalize(message);
    });

    return () => {
      void unsubStream.then((unsub) => unsub());
      void unsubComplete.then((unsub) => unsub());
      void unsubError.then((unsub) => unsub());
    };
  }, [projectId]);

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight, behavior: "smooth" });
  }, [messages.length, streamingContent]);

  const handleSubmit = useCallback(
    async (question: string) => {
      const userMessage: AskMessage = {
        id: crypto.randomUUID(),
        conversationId: "",
        role: "user",
        content: question,
        agent: null,
        model: null,
        createdAt: new Date().toISOString(),
      };
      const store = useAskStore.getState();
      store.addMessage(userMessage);
      store.setIsAsking(true);
      store.setStreamingContent("");
      store.setEditPrefill("");
      try {
        const messageId = await askQuestion(projectId, question, selectedAgent, selectedModel);
        useAskStore.getState().setStreamingMessageId(messageId);
      } catch {
        useAskStore.getState().setIsAsking(false);
      }
    },
    [projectId, selectedAgent, selectedModel],
  );

  const handleStop = useCallback(() => {
    void stopAsk(projectId);
    const store = useAskStore.getState();
    store.setIsAsking(false);
    store.setStreamingContent("");
    store.setStreamingMessageId(null);
  }, [projectId]);

  const handleCopy = useCallback((messageId: string) => {
    void copyAskMessage(messageId);
  }, []);

  const handleEdit = useCallback((messageId: string, content: string) => {
    void truncateAskFrom(projectId, messageId).then((truncated) => {
      const store = useAskStore.getState();
      store.setMessages(truncated);
      store.setEditPrefill(content);
    });
  }, [projectId]);

  const handleRetry = useCallback((messageId: string) => {
    const store = useAskStore.getState();
    store.setIsAsking(true);
    store.setStreamingContent("");
    void retryAsk(projectId, messageId, selectedAgent, selectedModel).then((newId) => {
      askHistory(projectId).then((history) => useAskStore.getState().setMessages(history));
      useAskStore.getState().setStreamingMessageId(newId);
    }).catch(() => useAskStore.getState().setIsAsking(false));
  }, [projectId, selectedAgent, selectedModel]);

  const handleRetryWith = useCallback((messageId: string, agent: string) => {
    const store = useAskStore.getState();
    store.setSelectedAgent(agent);
    store.setIsAsking(true);
    store.setStreamingContent("");
    void retryAsk(projectId, messageId, agent, null).then((newId) => {
      askHistory(projectId).then((history) => useAskStore.getState().setMessages(history));
      useAskStore.getState().setStreamingMessageId(newId);
    }).catch(() => useAskStore.getState().setIsAsking(false));
  }, [projectId]);

  if (disabled) {
    return (
      <div className="flex h-full items-center justify-center">
        <EmptyState icon={<MessageSquare className="h-8 w-8" />} title="Ask is available for active projects" />
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      <div ref={scrollRef} className="flex-1 overflow-y-auto px-4 py-4">
        {messages.length === 0 && !isAsking ? (
          <EmptyState icon={<MessageSquare className="h-8 w-8" />} title="Ask questions about your project" />
        ) : (
          <>
            {messages.map((msg) => (
              <AskMessageBubble
                key={msg.id}
                message={msg}
                onCopy={() => handleCopy(msg.id)}
                onEdit={() => handleEdit(msg.id, msg.content)}
                onRetry={() => handleRetry(msg.id)}
                onRetryWith={(agent) => handleRetryWith(msg.id, agent)}
              />
            ))}
            {isAsking ? (
              <div className="flex justify-start mb-3">
                <div className="max-w-[85%] rounded-md px-4 py-3 text-sm bg-surface border border-border text-text">
                  {streamingContent ? (
                    <ReactMarkdown remarkPlugins={[remarkGfm]} components={MARKDOWN_COMPONENTS}>
                      {streamingContent}
                    </ReactMarkdown>
                  ) : (
                    <span className="text-text-dim">Thinking...</span>
                  )}
                  <div className="mt-2">
                    <Badge variant="info">Streaming</Badge>
                  </div>
                </div>
              </div>
            ) : null}
          </>
        )}
      </div>
      <AskInput
        disabled={disabled}
        isAsking={isAsking}
        selectedAgent={selectedAgent}
        selectedModel={selectedModel}
        initialValue={editPrefill}
        onAgentChange={(agent) => useAskStore.getState().setSelectedAgent(agent)}
        onModelChange={(model) => useAskStore.getState().setSelectedModel(model)}
        onSubmit={(question) => { void handleSubmit(question); }}
        onStop={handleStop}
      />
    </div>
  );
}
