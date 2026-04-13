import type { Components } from "react-markdown";

type MarkdownDensity = "compact" | "comfortable";

function spacing(density: MarkdownDensity, compactValue: string, comfortableValue: string) {
  return density === "compact" ? compactValue : comfortableValue;
}

export function createMarkdownComponents(density: MarkdownDensity): Components {
  return {
    h1: ({ children }) => (
      <h1
        className={`border-border border-b pb-1 font-bold text-text ${spacing(density, "mt-3 mb-2 text-base", "mt-4 mb-3 text-xl")}`}
      >
        {children}
      </h1>
    ),
    h2: ({ children }) => (
      <h2
        className={`font-bold text-text ${spacing(density, "mt-3 mb-2 text-sm", "mt-4 mb-2 text-lg")}`}
      >
        {children}
      </h2>
    ),
    h3: ({ children }) => (
      <h3
        className={`font-bold text-text-muted ${spacing(density, "mt-2 mb-1 text-sm", "mt-3 mb-2 text-base")}`}
      >
        {children}
      </h3>
    ),
    p: ({ children }) => (
      <p
        className={`text-sm text-text leading-relaxed ${spacing(density, "mb-2 font-sans", "mb-3")}`}
      >
        {children}
      </p>
    ),
    ul: ({ children }) => (
      <ul className={`list-none space-y-1 pl-3 ${spacing(density, "mb-2", "mb-3")}`}>{children}</ul>
    ),
    ol: ({ children }) => (
      <ol
        className={`list-none space-y-1 pl-3 ${spacing(density, "mb-2", "counter-reset-item mb-3")}`}
      >
        {children}
      </ol>
    ),
    li: ({ children }) => (
      <li className={`flex gap-2 text-sm text-text ${density === "compact" ? "font-sans" : ""}`}>
        <span className="mt-0.5 shrink-0 text-primary">›</span>
        <span>{children}</span>
      </li>
    ),
    code: ({ children, className }) => {
      const isBlock = className?.includes("language-");
      if (isBlock) {
        return (
          <code
            className={`block overflow-x-auto rounded-md border border-border bg-surface font-mono text-primary text-xs ${spacing(density, "mb-2 px-3 py-2", "mb-3 px-4 py-3")}`}
          >
            {children}
          </code>
        );
      }
      return (
        <code
          className={`rounded border border-border bg-elevated font-mono text-cyan text-xs ${spacing(density, "px-1 py-0.5", "px-1.5 py-0.5")}`}
        >
          {children}
        </code>
      );
    },
    pre: ({ children }) => <div className={spacing(density, "mb-2", "mb-3")}>{children}</div>,
    blockquote: ({ children }) => (
      <blockquote
        className={`border-border border-l-2 text-sm text-text-muted italic ${spacing(density, "mb-2 pl-3", "mb-3 pl-4")}`}
      >
        {children}
      </blockquote>
    ),
    strong: ({ children }) => <strong className="font-bold text-text">{children}</strong>,
    em: ({ children }) => <em className="text-text-muted italic">{children}</em>,
    a: ({ children, href }) => (
      <a
        href={href}
        className="text-cyan underline transition-colors hover:text-cyan/80"
        target="_blank"
        rel="noopener noreferrer"
      >
        {children}
      </a>
    ),
    hr: () => <hr className={`border-border ${spacing(density, "my-3", "my-4")}`} />,
    table: ({ children }) => (
      <div className={`overflow-x-auto ${spacing(density, "mb-2", "mb-3")}`}>
        <table className="w-full rounded-md border border-border text-sm">{children}</table>
      </div>
    ),
    thead: ({ children }) => <thead className="bg-elevated">{children}</thead>,
    th: ({ children }) => (
      <th
        className={`border-border border-b text-left font-sans text-text-muted text-xs uppercase tracking-wider ${spacing(density, "px-2 py-1.5", "px-3 py-2")}`}
      >
        {children}
      </th>
    ),
    td: ({ children }) => (
      <td
        className={`border-border/50 border-b text-text ${spacing(density, "px-2 py-1.5 text-sm", "px-3 py-2")}`}
      >
        {children}
      </td>
    ),
  };
}

export const MARKDOWN_COMPONENTS = createMarkdownComponents("compact");

export const MARKDOWN_COMPONENTS_COMFORTABLE = createMarkdownComponents("comfortable");
