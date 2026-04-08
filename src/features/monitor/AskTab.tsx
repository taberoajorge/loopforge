import { useCallback, useEffect, useRef } from "react";
import { MessageSquare } from "lucide-react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { EmptyState } from "../../components/EmptyState";
import { MARKDOWN_COMPONENTS } from "../../components/markdownComponents";
import { Badge } from "../../components/ui";
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

type AskTabProps = {
  projectId: string;
  disabled: boolean;
};

export function AskTab({ projectId, disabled }: AskTabProps) {
  const store = useAskStore();
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    store.switchProject(projectId);
    askHistory(projectId)
      .then((messages) => store.setMessages(messages))
      .catch(() => {});
  }, [projectId]);

  useEffect(() => {
    const unsubStream = onAskStream((payload) => {
      if (payload.projectId !== projectId) return;
      store.appendStreamChunk(payload.chunk);
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
      store.finalize(message);
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
      store.finalize(message);
    });

    return () => {
      void unsubStream.then((unsub) => unsub());
      void unsubComplete.then((unsub) => unsub());
      void unsubError.then((unsub) => unsub());
    };
  }, [projectId]);

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight, behavior: "smooth" });
  }, [store.messages.length, store.streamingContent]);

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
      store.addMessage(userMessage);
      store.setIsAsking(true);
      store.setStreamingContent("");
      store.setEditPrefill("");
      try {
        const messageId = await askQuestion(projectId, question, store.selectedAgent, store.selectedModel);
        store.setStreamingMessageId(messageId);
      } catch {
        store.setIsAsking(false);
      }
    },
    [projectId, store.selectedAgent, store.selectedModel],
  );

  const handleStop = useCallback(() => {
    void stopAsk(projectId);
    store.setIsAsking(false);
    store.setStreamingContent("");
    store.setStreamingMessageId(null);
  }, [projectId]);

  const handleCopy = useCallback((messageId: string) => {
    void copyAskMessage(messageId);
  }, []);

  const handleEdit = useCallback((messageId: string, content: string) => {
    void truncateAskFrom(projectId, messageId).then((messages) => {
      store.setMessages(messages);
      store.setEditPrefill(content);
    });
  }, [projectId]);

  const handleRetry = useCallback((messageId: string) => {
    store.setIsAsking(true);
    store.setStreamingContent("");
    void retryAsk(projectId, messageId, store.selectedAgent, store.selectedModel).then((newId) => {
      askHistory(projectId).then((messages) => store.setMessages(messages));
      store.setStreamingMessageId(newId);
    }).catch(() => store.setIsAsking(false));
  }, [projectId, store.selectedAgent, store.selectedModel]);

  const handleRetryWith = useCallback((messageId: string, agent: string) => {
    store.setSelectedAgent(agent);
    store.setIsAsking(true);
    store.setStreamingContent("");
    void retryAsk(projectId, messageId, agent, null).then((newId) => {
      askHistory(projectId).then((messages) => store.setMessages(messages));
      store.setStreamingMessageId(newId);
    }).catch(() => store.setIsAsking(false));
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
        {store.messages.length === 0 && !store.isAsking ? (
          <EmptyState icon={<MessageSquare className="h-8 w-8" />} title="Ask questions about your project" />
        ) : (
          <>
            {store.messages.map((msg) => (
              <AskMessageBubble
                key={msg.id}
                message={msg}
                onCopy={() => handleCopy(msg.id)}
                onEdit={() => handleEdit(msg.id, msg.content)}
                onRetry={() => handleRetry(msg.id)}
                onRetryWith={(agent) => handleRetryWith(msg.id, agent)}
              />
            ))}
            {store.isAsking ? (
              <div className="flex justify-start mb-3">
                <div className="max-w-[85%] rounded-md px-4 py-3 text-sm bg-surface border border-border text-text">
                  {store.streamingContent ? (
                    <ReactMarkdown remarkPlugins={[remarkGfm]} components={MARKDOWN_COMPONENTS}>
                      {store.streamingContent}
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
        isAsking={store.isAsking}
        selectedAgent={store.selectedAgent}
        selectedModel={store.selectedModel}
        initialValue={store.editPrefill}
        onAgentChange={store.setSelectedAgent}
        onModelChange={store.setSelectedModel}
        onSubmit={(question) => { void handleSubmit(question); }}
        onStop={handleStop}
      />
    </div>
  );
}
