import { useEffect, useRef, useState } from "react";
import { ephemeralQuery, type EphemeralAnswer } from "../lib/tauri";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "./ui/dialog";
import { Input } from "./ui/input";

interface EphemeralOverlayProps {
  projectId: string;
  isOpen: boolean;
  onClose: () => void;
}

export function EphemeralOverlay({
  projectId,
  isOpen,
  onClose,
}: EphemeralOverlayProps) {
  const [question, setQuestion] = useState("");
  const [answer, setAnswer] = useState<EphemeralAnswer | null>(null);
  const [loading, setLoading] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const previousFocusRef = useRef<HTMLElement | null>(null);

  useEffect(() => {
    if (!isOpen) {
      return;
    }
    const activeElement = document.activeElement;
    previousFocusRef.current = activeElement instanceof HTMLElement ? activeElement : null;
  }, [isOpen]);

  function handleOpenChange(open: boolean) {
    if (!open) {
      onClose();
    }
  }

  async function handleSubmit(evt: React.FormEvent) {
    evt.preventDefault();
    if (!question.trim() || loading) return;

    setLoading(true);
    setAnswer(null);

    try {
      const result = await ephemeralQuery(projectId, question.trim());
      setAnswer(result);
    } catch {
      setAnswer({
        question,
        answer: "Failed to get answer. Is a loop running?",
        source: "instant",
      });
    } finally {
      setLoading(false);
    }
  }

  function clearAnswer() {
    setQuestion("");
    setAnswer(null);
    inputRef.current?.focus();
  }

  return (
    <Dialog open={isOpen} onOpenChange={handleOpenChange}>
      <DialogContent
        className="w-[min(36rem,calc(100vw-2rem))] p-0 overflow-hidden"
        onOpenAutoFocus={(event) => {
          event.preventDefault();
          inputRef.current?.focus();
        }}
        onCloseAutoFocus={(event) => {
          event.preventDefault();
          previousFocusRef.current?.focus();
        }}
      >
        <DialogHeader className="px-4 pt-4 pb-2">
          <DialogTitle className="font-mono text-base">Ephemeral Query</DialogTitle>
          <DialogDescription>
            Ask about the current session without leaving Monitor.
          </DialogDescription>
        </DialogHeader>

        <form id="ephemeral-query-form" onSubmit={handleSubmit} className="px-4 pb-3 space-y-2">
          <Input
            ref={inputRef}
            type="text"
            value={question}
            onChange={(event) => setQuestion(event.target.value)}
            placeholder="Ask about the current session..."
            className="font-mono text-sm"
          />
          {loading ? <p className="text-xs font-mono text-text-dim">thinking...</p> : null}
        </form>

        {answer && (
          <div className="border-y border-border px-4 py-3">
            <div className="mb-2 flex items-center gap-2">
              <Badge
                variant={answer.source === "instant" ? "info" : "neutral"}
                className="font-mono uppercase tracking-wide"
              >
                {answer.source}
              </Badge>
            </div>
            <pre className="whitespace-pre-wrap text-sm font-mono leading-relaxed text-text">
              {answer.answer}
            </pre>
          </div>
        )}

        <DialogFooter className="border-t border-border/50 px-4 py-3 sm:justify-between">
          <p className="text-xs font-mono text-text-dim">Esc to close · Ctrl+Shift+Space to toggle</p>
          <div className="flex items-center gap-2">
            <Button variant="secondary" size="sm" onClick={clearAnswer} disabled={loading}>
              Clear
            </Button>
            <Button variant="ghost" size="sm" onClick={onClose}>
              Dismiss
            </Button>
            <Button
              form="ephemeral-query-form"
              type="submit"
              variant="primary"
              size="sm"
              disabled={loading || !question.trim()}
            >
              Ask
            </Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
