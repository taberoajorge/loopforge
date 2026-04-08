import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import type { Components } from "react-markdown";
import { Badge, Card, CardContent, CardHeader, CardTitle, ScrollArea, ScrollViewport, Separator } from "./ui";
import { cn } from "../lib/utils";

const MARKDOWN_COMPONENTS: Components = {
  h1: ({ children }) => (
    <h1 className="text-xl font-bold text-text mb-3 mt-4 border-b border-border pb-1">
      {children}
    </h1>
  ),
  h2: ({ children }) => (
    <h2 className="text-lg font-bold text-text mb-2 mt-4">{children}</h2>
  ),
  h3: ({ children }) => (
    <h3 className="text-base font-bold text-text-muted mb-2 mt-3">{children}</h3>
  ),
  p: ({ children }) => (
    <p className="text-text text-sm mb-3 leading-relaxed">{children}</p>
  ),
  ul: ({ children }) => (
    <ul className="list-none space-y-1 mb-3 pl-3">{children}</ul>
  ),
  ol: ({ children }) => (
    <ol className="list-none space-y-1 mb-3 pl-3 counter-reset-item">{children}</ol>
  ),
  li: ({ children }) => (
    <li className="text-text text-sm flex gap-2">
      <span className="text-primary shrink-0 mt-0.5">›</span>
      <span>{children}</span>
    </li>
  ),
  code: ({ children, className }) => {
    const isBlock = className?.includes("language-");
    if (isBlock) {
      return (
        <code className="block bg-surface border border-border rounded-md px-4 py-3 text-xs font-mono text-primary overflow-x-auto mb-3">
          {children}
        </code>
      );
    }
    return (
      <code className="bg-elevated border border-border rounded px-1.5 py-0.5 text-xs font-mono text-cyan">
        {children}
      </code>
    );
  },
  pre: ({ children }) => <div className="mb-3">{children}</div>,
  blockquote: ({ children }) => (
    <blockquote className="border-l-2 border-border pl-4 text-text-muted text-sm italic mb-3">
      {children}
    </blockquote>
  ),
  strong: ({ children }) => (
    <strong className="font-bold text-text">{children}</strong>
  ),
  em: ({ children }) => (
    <em className="italic text-text-muted">{children}</em>
  ),
  a: ({ children, href }) => (
    <a
      href={href}
      className="text-cyan underline hover:text-cyan/80 transition-colors"
      target="_blank"
      rel="noopener noreferrer"
    >
      {children}
    </a>
  ),
  hr: () => <hr className="border-border my-4" />,
  table: ({ children }) => (
    <div className="overflow-x-auto mb-3">
      <table className="w-full text-sm border border-border rounded-md">
        {children}
      </table>
    </div>
  ),
  thead: ({ children }) => (
    <thead className="bg-elevated">{children}</thead>
  ),
  th: ({ children }) => (
    <th className="px-3 py-2 text-left text-text-muted font-sans text-xs uppercase tracking-wider border-b border-border">
      {children}
    </th>
  ),
  td: ({ children }) => (
    <td className="px-3 py-2 text-text border-b border-border/50">{children}</td>
  ),
};

interface MarkdownPreviewProps {
  content: string;
  className?: string;
}

export function MarkdownPreview({ content, className = "" }: MarkdownPreviewProps) {
  return (
    <Card variant="ghost" className={cn("flex h-full min-h-0 flex-col", className)}>
      <CardHeader className="gap-2 border-0 p-3">
        <div className="flex items-center gap-2">
          <CardTitle className="text-xs font-sans uppercase tracking-widest text-text-muted">Markdown Preview</CardTitle>
          <Badge variant="info" className="font-mono">LIVE</Badge>
        </div>
        <Separator tone="muted" />
      </CardHeader>
      <CardContent className="min-h-0 flex-1 p-0">
        <ScrollArea className="h-full">
          <ScrollViewport padding="md" className="h-full">
            <div className="font-mono text-sm">
              <ReactMarkdown remarkPlugins={[remarkGfm]} components={MARKDOWN_COMPONENTS}>
                {content}
              </ReactMarkdown>
            </div>
          </ScrollViewport>
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
