import { useEffect, useState } from "react";
import { Send, Square } from "lucide-react";
import { Button, Field, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../../components/ui";
import { getAgentCapabilities, type AgentCapabilities } from "../../../lib/tauri";

const AGENT_NAMES = ["cursor", "codex", "claude", "gemini", "opencode"];

type AskInputProps = {
  disabled: boolean;
  isAsking: boolean;
  selectedAgent: string;
  selectedModel: string | null;
  initialValue?: string;
  onAgentChange: (agent: string) => void;
  onModelChange: (model: string | null) => void;
  onSubmit: (question: string) => void;
  onStop: () => void;
};

export function AskInput({
  disabled,
  isAsking,
  selectedAgent,
  selectedModel,
  initialValue,
  onAgentChange,
  onModelChange,
  onSubmit,
  onStop,
}: AskInputProps) {
  const [question, setQuestion] = useState("");

  useEffect(() => {
    if (initialValue !== undefined && initialValue !== "") {
      setQuestion(initialValue);
    }
  }, [initialValue]);
  const [capabilities, setCapabilities] = useState<AgentCapabilities | null>(null);

  useEffect(() => {
    let cancelled = false;
    getAgentCapabilities(selectedAgent)
      .then((caps) => {
        if (cancelled) return;
        setCapabilities(caps);
        if (caps.supportsModel) {
          const valid = caps.models.some((entry) => entry.id === selectedModel);
          if (!valid) onModelChange(caps.defaultModel ?? caps.models[0]?.id ?? null);
        } else {
          onModelChange(null);
        }
      })
      .catch(() => setCapabilities(null));
    return () => { cancelled = true; };
  }, [selectedAgent]);

  function handleSubmit() {
    const trimmed = question.trim();
    if (!trimmed || disabled || isAsking) return;
    onSubmit(trimmed);
    setQuestion("");
  }

  function handleKeyDown(event: React.KeyboardEvent<HTMLTextAreaElement>) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      handleSubmit();
    }
  }

  const showModel = capabilities !== null && capabilities.supportsModel;
  const loading = capabilities === null;

  return (
    <div className="border-t border-border bg-surface/40 px-4 py-3">
      <div className="grid grid-cols-[9rem_1fr] items-end gap-2 mb-2">
        <Field label="Agent">
          <Select value={selectedAgent} onValueChange={onAgentChange} disabled={isAsking}>
            <SelectTrigger><SelectValue /></SelectTrigger>
            <SelectContent>
              {AGENT_NAMES.map((name) => (
                <SelectItem key={name} value={name}>{name}</SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
        <Field label="Model" className={showModel ? "" : "invisible"}>
          <Select
            value={selectedModel ?? ""}
            onValueChange={(val) => onModelChange(val || null)}
            disabled={isAsking || loading}
          >
            <SelectTrigger><SelectValue placeholder={loading ? "Loading..." : "—"} /></SelectTrigger>
            <SelectContent>
              {(capabilities?.models ?? []).map((model) => (
                <SelectItem key={model.id} value={model.id}>{model.label}</SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
      </div>
      <div className="flex items-end gap-2">
        <textarea
          value={question}
          onChange={(event) => setQuestion(event.target.value)}
          onKeyDown={handleKeyDown}
          disabled={disabled || isAsking}
          placeholder="Ask about your project..."
          rows={2}
          className="flex-1 resize-none rounded-md border border-border bg-void px-3 py-2 text-sm text-text placeholder:text-text-dim focus:outline-none focus:ring-1 focus:ring-primary disabled:opacity-50"
        />
        {isAsking ? (
          <Button variant="secondary" size="md" onClick={onStop}>
            <Square className="h-3.5 w-3.5" />
          </Button>
        ) : (
          <Button variant="primary" size="md" onClick={handleSubmit} disabled={disabled || !question.trim()}>
            <Send className="h-3.5 w-3.5" />
          </Button>
        )}
      </div>
    </div>
  );
}
