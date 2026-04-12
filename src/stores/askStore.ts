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
  editPrefill: string;
  switchProject: (projectId: string) => void;
  setMessages: (messages: AskMessage[]) => void;
  addMessage: (message: AskMessage) => void;
  setStreamingContent: (content: string) => void;
  appendStreamChunk: (chunk: string) => void;
  setStreamingMessageId: (messageId: string | null) => void;
  setIsAsking: (asking: boolean) => void;
  setSelectedAgent: (agent: string) => void;
  setSelectedModel: (model: string | null) => void;
  setEditPrefill: (value: string) => void;
  finalize: (message: AskMessage) => void;
  reset: () => void;
}

const INITIAL_STATE = {
  activeProjectId: null as string | null,
  messages: [] as AskMessage[],
  streamingContent: "",
  streamingMessageId: null as string | null,
  isAsking: false,
  selectedAgent: "claude",
  selectedModel: null as string | null,
  editPrefill: "",
};

export const useAskStore = create<AskState>()((set, get) => ({
  ...INITIAL_STATE,
  switchProject: (projectId) => {
    if (get().activeProjectId === projectId) return;
    set({ ...INITIAL_STATE, activeProjectId: projectId, selectedAgent: get().selectedAgent });
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
  reset: () => set(INITIAL_STATE),
}));
