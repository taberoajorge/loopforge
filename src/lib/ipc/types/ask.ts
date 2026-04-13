export interface AskMessage {
  id: string;
  conversationId: string;
  role: "user" | "assistant";
  content: string;
  agent: string | null;
  model: string | null;
  createdAt: string;
}

export interface AskStreamPayload {
  projectId: string;
  messageId: string;
  chunk: string;
}

export interface AskQuestionResult {
  messageId: string;
  userMessage: AskMessage;
}

export interface AskCompletePayload {
  projectId: string;
  messageId: string;
  fullContent: string;
  agent: string;
  model: string | null;
  message: AskMessage;
}

export interface AskErrorPayload {
  projectId: string;
  messageId: string;
  error: string;
  message: AskMessage;
}
