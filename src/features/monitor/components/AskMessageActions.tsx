import { ArrowRightLeft, Check, Copy, Pencil, RefreshCw } from "lucide-react";
import { useEffect, useState } from "react";
import { Button } from "../../../components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "../../../components/ui/dropdown-menu";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "../../../components/ui/tooltip";
import { getKnownAgents } from "../../../lib/tauri";

type AskMessageActionsProps = {
  role: "user" | "assistant";
  onCopy: () => void;
  onEdit?: () => void;
  onRetry?: () => void;
  onRetryWith?: (agent: string) => void;
};

export function AskMessageActions({
  role,
  onCopy,
  onEdit,
  onRetry,
  onRetryWith,
}: AskMessageActionsProps) {
  const [copied, setCopied] = useState(false);
  const [knownAgents, setKnownAgents] = useState<string[]>([]);

  useEffect(() => {
    let cancelled = false;
    getKnownAgents()
      .then((agentNames) => {
        if (cancelled) return;
        setKnownAgents(agentNames);
      })
      .catch(() => {
        if (cancelled) return;
        setKnownAgents([]);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  function handleCopy() {
    onCopy();
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  }

  return (
    <TooltipProvider delayDuration={300}>
      <div className="flex items-center gap-0.5 pt-1">
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="sm"
              className="h-6 w-6 p-0 text-text-dim hover:text-text"
              onClick={handleCopy}
            >
              {copied ? <Check className="h-3 w-3 text-primary" /> : <Copy className="h-3 w-3" />}
            </Button>
          </TooltipTrigger>
          <TooltipContent side="bottom">{copied ? "Copied" : "Copy"}</TooltipContent>
        </Tooltip>
        {role === "user" && onEdit ? (
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="sm"
                className="h-6 w-6 p-0 text-text-dim hover:text-text"
                onClick={onEdit}
              >
                <Pencil className="h-3 w-3" />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="bottom">Edit</TooltipContent>
          </Tooltip>
        ) : null}
        {role === "user" && onRetry ? (
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="sm"
                className="h-6 w-6 p-0 text-text-dim hover:text-text"
                onClick={onRetry}
              >
                <RefreshCw className="h-3 w-3" />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="bottom">Retry</TooltipContent>
          </Tooltip>
        ) : null}
        {role === "user" && onRetryWith && knownAgents.length > 0 ? (
          <DropdownMenu>
            <Tooltip>
              <TooltipTrigger asChild>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-6 w-6 p-0 text-text-dim hover:text-text"
                  >
                    <ArrowRightLeft className="h-3 w-3" />
                  </Button>
                </DropdownMenuTrigger>
              </TooltipTrigger>
              <TooltipContent side="bottom">Retry with...</TooltipContent>
            </Tooltip>
            <DropdownMenuContent align="start">
              {knownAgents.map((name) => (
                <DropdownMenuItem key={name} onClick={() => onRetryWith(name)}>
                  {name}
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>
        ) : null}
      </div>
    </TooltipProvider>
  );
}
