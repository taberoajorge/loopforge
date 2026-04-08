import { create } from "zustand";
import type { AskMessage } from "../lib/tauri";

interface AskState {
  activeProjectId: string | null;
  messages: AskMessage[];
  streamingContent: string;
  streamingMessageId: string | null;
  isAsking: boolean;
  selectedAgent: string;
  selectedModel: string | null;
  switchProject: (projectId: string) => void;
  setMessages: (messages: AskMessage[]) => void;
  addMessage: (message: AskMessage) => void;
  setStreamingContent: (content: string) => void;
  appendStreamChunk: (chunk: string) => void;
  setStreamingMessageId: (messageId: string | null) => void;
  setIsAsking: (asking: boolean) => void;
  setSelectedAgent: (agent: string) => void;
  setSelectedModel: (model: string | null) => void;
  editPrefill: string;
  setEditPrefill: (value: string) => void;
  finalize: (message: AskMessage) => void;
  reset: () => void;
}

export const useAskStore = create<AskState>()((set, get) => ({
  activeProjectId: null,
  messages: [],
  streamingContent: "",
  streamingMessageId: null,
  isAsking: false,
  selectedAgent: "claude",
  selectedModel: null,
  editPrefill: "",
  switchProject: (projectId) => {
    if (get().activeProjectId === projectId) return;
    set({
      activeProjectId: projectId,
      messages: [],
      streamingContent: "",
      streamingMessageId: null,
      isAsking: false,
    });
  },
  setMessages: (messages) => set({ messages }),
  addMessage: (message) =>
    set((state) => ({ messages: [...state.messages, message] })),
  setStreamingContent: (content) => set({ streamingContent: content }),
  appendStreamChunk: (chunk) =>
    set((state) => ({ streamingContent: state.streamingContent + chunk })),
  setStreamingMessageId: (messageId) => set({ streamingMessageId: messageId }),
  setIsAsking: (asking) => set({ isAsking: asking }),
  setSelectedAgent: (agent) => set({ selectedAgent: agent }),
  setSelectedModel: (model) => set({ selectedModel: model }),
  setEditPrefill: (value) => set({ editPrefill: value }),
  finalize: (message) =>
    set((state) => ({
      messages: [...state.messages, message],
      streamingContent: "",
      streamingMessageId: null,
      isAsking: false,
    })),
  reset: () =>
    set({
      activeProjectId: null,
      messages: [],
      streamingContent: "",
      streamingMessageId: null,
      isAsking: false,
    }),
}));
