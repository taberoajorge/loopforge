import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { cn } from "../lib/utils";
import { MARKDOWN_COMPONENTS_COMFORTABLE } from "./markdownComponents";
import { Badge } from "./ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "./ui/card";
import { ScrollArea, ScrollViewport } from "./ui/scroll-area";
import { Separator } from "./ui/separator";

interface MarkdownPreviewProps {
  content: string;
  className?: string;
}

export function MarkdownPreview({ content, className = "" }: MarkdownPreviewProps) {
  return (
    <Card variant="ghost" className={cn("flex h-full min-h-0 flex-col", className)}>
      <CardHeader className="gap-2 border-0 p-3">
        <div className="flex items-center gap-2">
          <CardTitle className="font-sans text-text-muted text-xs uppercase tracking-widest">
            Markdown Preview
          </CardTitle>
          <Badge variant="info" className="font-mono">
            LIVE
          </Badge>
        </div>
        <Separator tone="muted" />
      </CardHeader>
      <CardContent className="min-h-0 flex-1 p-0">
        <ScrollArea className="h-full">
          <ScrollViewport padding="md" className="h-full">
            <div className="font-mono text-sm">
              <ReactMarkdown
                remarkPlugins={[remarkGfm]}
                components={MARKDOWN_COMPONENTS_COMFORTABLE}
              >
                {content}
              </ReactMarkdown>
            </div>
          </ScrollViewport>
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
