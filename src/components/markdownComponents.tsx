import type { Components } from "react-markdown";

export const MARKDOWN_COMPONENTS: Components = {
  h1: ({ children }) => (
    <h1 className="text-base font-bold text-text mb-2 mt-3 border-b border-border pb-1">
      {children}
    </h1>
  ),
  h2: ({ children }) => (
    <h2 className="text-sm font-bold text-text mb-2 mt-3">{children}</h2>
  ),
  h3: ({ children }) => (
    <h3 className="text-sm font-bold text-text-muted mb-1 mt-2">{children}</h3>
  ),
  p: ({ children }) => (
    <p className="text-text text-sm mb-2 leading-relaxed font-sans">{children}</p>
  ),
  ul: ({ children }) => (
    <ul className="list-none space-y-1 mb-2 pl-3">{children}</ul>
  ),
  ol: ({ children }) => (
    <ol className="list-none space-y-1 mb-2 pl-3">{children}</ol>
  ),
  li: ({ children }) => (
    <li className="text-text text-sm flex gap-2 font-sans">
      <span className="text-primary shrink-0 mt-0.5">›</span>
      <span>{children}</span>
    </li>
  ),
  code: ({ children, className }) => {
    const isBlock = className?.includes("language-");
    if (isBlock) {
      return (
        <code className="block bg-surface border border-border rounded-md px-3 py-2 text-xs font-mono text-primary overflow-x-auto mb-2">
          {children}
        </code>
      );
    }
    return (
      <code className="bg-elevated border border-border rounded px-1 py-0.5 text-xs font-mono text-cyan">
        {children}
      </code>
    );
  },
  pre: ({ children }) => <div className="mb-2">{children}</div>,
  blockquote: ({ children }) => (
    <blockquote className="border-l-2 border-border pl-3 text-text-muted text-sm italic mb-2">
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
  hr: () => <hr className="border-border my-3" />,
  table: ({ children }) => (
    <div className="overflow-x-auto mb-2">
      <table className="w-full text-sm border border-border rounded-md">
        {children}
      </table>
    </div>
  ),
  thead: ({ children }) => (
    <thead className="bg-elevated">{children}</thead>
  ),
  th: ({ children }) => (
    <th className="px-2 py-1.5 text-left text-text-muted font-sans text-xs uppercase tracking-wider border-b border-border">
      {children}
    </th>
  ),
  td: ({ children }) => (
    <td className="px-2 py-1.5 text-text text-sm border-b border-border/50">{children}</td>
  ),
};
