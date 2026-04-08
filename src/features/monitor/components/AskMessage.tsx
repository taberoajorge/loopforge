import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { MARKDOWN_COMPONENTS } from "../../../components/markdownComponents";
import { Badge } from "../../../components/ui/badge";
import type { AskMessage as AskMessageType } from "../../../lib/tauri";
import { AskMessageActions } from "./AskMessageActions";

type AskMessageProps = {
  message: AskMessageType;
  onCopy: () => void;
  onEdit?: () => void;
  onRetry?: () => void;
  onRetryWith?: (agent: string) => void;
};

export function AskMessageBubble({ message, onCopy, onEdit, onRetry, onRetryWith }: AskMessageProps) {
  const isUser = message.role === "user";

  return (
    <div className={`group flex ${isUser ? "justify-end" : "justify-start"} mb-3`}>
      <div className="max-w-[85%]">
        <div
          className={`rounded-md px-4 py-3 text-sm ${
            isUser
              ? "bg-primary/10 border border-primary/20 text-text"
              : "bg-surface border border-border text-text"
          }`}
        >
          {isUser ? (
            <p className="whitespace-pre-wrap">{message.content}</p>
          ) : (
            <ReactMarkdown remarkPlugins={[remarkGfm]} components={MARKDOWN_COMPONENTS}>
              {message.content}
            </ReactMarkdown>
          )}
          {!isUser && message.agent ? (
            <div className="mt-2 flex items-center gap-2">
              <Badge variant="neutral">{message.agent}</Badge>
              {message.model ? (
                <span className="text-text-dim text-[10px]">{message.model}</span>
              ) : null}
            </div>
          ) : null}
        </div>
        <div className="opacity-0 group-hover:opacity-100 transition-opacity">
          <AskMessageActions
            role={message.role}
            onCopy={onCopy}
            onEdit={isUser ? onEdit : undefined}
            onRetry={isUser ? onRetry : undefined}
            onRetryWith={isUser ? onRetryWith : undefined}
          />
        </div>
      </div>
    </div>
  );
}
