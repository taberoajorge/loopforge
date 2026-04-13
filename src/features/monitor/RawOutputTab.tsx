import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import { TerminalSquare } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import "@xterm/xterm/css/xterm.css";
import { EmptyState } from "../../components/EmptyState";
import { Button } from "../../components/ui/button";
import { reportError } from "../../lib/reportError";
import { loadOutputLog, onAgentOutput } from "../../lib/tauri";
import { useDisplayVocabularyStore } from "../../stores/displayVocabularyStore";
import { TerminalFrame } from "./components/TerminalFrame";
import { useTerminalFit } from "./useTerminalFit";
import { buildXtermTheme } from "./xtermTheme";

export function RawOutputTab({ projectId }: { projectId: string }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const outputBufferRef = useRef<string[]>([]);
  const atBottomRef = useRef(true);
  const [atBottom, setAtBottom] = useState(true);
  const [copyState, setCopyState] = useState<"idle" | "copied" | "failed">("idle");
  const [lineCount, setLineCount] = useState(0);
  const maxOutputLines = useDisplayVocabularyStore(
    (state) => state.vocabulary?.maxOutputLines ?? 5000,
  );

  useTerminalFit({ containerRef, fitAddonRef });

  useEffect(() => {
    if (!containerRef.current) return;

    const terminal = new Terminal({
      theme: buildXtermTheme(),
      fontFamily: '"JetBrains Mono", "Fira Code", "Cascadia Code", monospace',
      fontSize: 12,
      lineHeight: 1.4,
      cursorStyle: "bar",
      cursorBlink: false,
      scrollback: maxOutputLines,
      disableStdin: true,
    });

    const fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    terminal.open(containerRef.current);
    fitAddon.fit();

    terminalRef.current = terminal;
    fitAddonRef.current = fitAddon;

    for (const outputLine of outputBufferRef.current) {
      terminal.writeln(outputLine);
    }
    terminal.scrollToBottom();

    terminal.onScroll(() => {
      const viewportRows = terminal.rows;
      const totalLines = terminal.buffer.active.length;
      const scrollTop = terminal.buffer.active.viewportY;
      const isAtBottom = scrollTop + viewportRows >= totalLines;
      atBottomRef.current = isAtBottom;
      setAtBottom(isAtBottom);
    });

    return () => {
      terminal.dispose();
      terminalRef.current = null;
      fitAddonRef.current = null;
    };
  }, [maxOutputLines]);

  useEffect(() => {
    if (terminalRef.current) {
      terminalRef.current.options.theme = buildXtermTheme();
    }
  }, []);

  useEffect(() => {
    if (copyState === "idle") {
      return;
    }
    const timeoutId = setTimeout(() => {
      setCopyState("idle");
    }, 1400);
    return () => clearTimeout(timeoutId);
  }, [copyState]);

  useEffect(() => {
    outputBufferRef.current = [];
    terminalRef.current?.clear();

    void loadOutputLog(projectId)
      .then((content) => {
        if (!content.trim()) {
          setLineCount(0);
          return;
        }
        const lines = content.split("\n");
        outputBufferRef.current = lines.slice(-maxOutputLines);
        setLineCount(outputBufferRef.current.length);
        if (terminalRef.current) {
          terminalRef.current.clear();
          for (const outputLine of outputBufferRef.current) {
            terminalRef.current.writeln(outputLine);
          }
          terminalRef.current.scrollToBottom();
        }
      })
      .catch((caughtError: unknown) => {
        reportError("RawOutputTab.loadOutputLog", caughtError);
      });

    const unlistenPromise = onAgentOutput((payload) => {
      if (payload.projectId !== projectId) {
        return;
      }

      outputBufferRef.current.push(payload.line);
      if (outputBufferRef.current.length > maxOutputLines) {
        outputBufferRef.current.shift();
      }
      setLineCount(outputBufferRef.current.length);

      terminalRef.current?.writeln(payload.line);
      if (atBottomRef.current) {
        terminalRef.current?.scrollToBottom();
      }
    });
    return () => {
      unlistenPromise.then((fn) => fn());
    };
  }, [projectId, maxOutputLines]);

  function jumpToBottom() {
    terminalRef.current?.scrollToBottom();
    atBottomRef.current = true;
    setAtBottom(true);
  }

  function clearOutput() {
    outputBufferRef.current = [];
    terminalRef.current?.clear();
    setLineCount(0);
    atBottomRef.current = true;
    setAtBottom(true);
  }

  async function copyOutput() {
    const output = outputBufferRef.current.join("\n");
    if (!output.trim()) {
      setCopyState("failed");
      return;
    }
    try {
      await navigator.clipboard.writeText(output);
      setCopyState("copied");
    } catch {
      setCopyState("failed");
    }
  }

  const copyLabel =
    copyState === "copied" ? "Copied" : copyState === "failed" ? "Copy failed" : "Copy";

  return (
    <TerminalFrame
      controls={
        <>
          <Button variant="outline" size="sm" onClick={copyOutput}>
            {copyLabel}
          </Button>
          <Button variant="outline" size="sm" onClick={clearOutput}>
            Clear
          </Button>
          {!atBottom ? (
            <Button variant="secondary" size="sm" onClick={jumpToBottom}>
              Jump to bottom
            </Button>
          ) : null}
        </>
      }
    >
      <div className="relative h-full overflow-hidden rounded-md border border-border/60 bg-void p-2">
        <div ref={containerRef} className="h-full overflow-hidden" />
        {lineCount === 0 ? (
          <div className="absolute inset-2 flex items-center justify-center bg-void/95">
            <EmptyState
              title="No output yet"
              description="Run the loop to stream agent output here."
              icon={<TerminalSquare className="h-4 w-4" />}
              compact
              padding="tight"
            />
          </div>
        ) : null}
      </div>
    </TerminalFrame>
  );
}
