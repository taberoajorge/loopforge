import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";

const emptyVariants = cva(
  "flex flex-col items-center justify-center rounded-lg border border-dashed px-6 py-8 text-center",
  {
    variants: {
      tone: {
        default: "border-border bg-surface/60",
        elevated: "border-border bg-elevated/70",
      },
      compact: {
        true: "gap-2 px-4 py-5",
        false: "gap-3",
      },
    },
    defaultVariants: {
      tone: "default",
      compact: false,
    },
  },
);

export type EmptyProps = React.HTMLAttributes<HTMLDivElement> &
  VariantProps<typeof emptyVariants> & {
    title: React.ReactNode;
    description?: React.ReactNode;
    icon?: React.ReactNode;
    action?: React.ReactNode;
    align?: "center" | "start";
    padding?: "default" | "tight";
  };

function Empty({
  align = "center",
  action,
  className,
  compact,
  description,
  icon,
  padding = "default",
  title,
  tone,
  ...props
}: EmptyProps) {
  return (
    <div
      className={cn(
        emptyVariants({ tone, compact }),
        align === "start" ? "items-start text-left" : "",
        padding === "tight" ? "px-4 py-4" : "",
        className,
      )}
      {...props}
    >
      {icon ? <div className="text-text-dim">{icon}</div> : null}
      <div className={cn("space-y-1", align === "start" ? "max-w-full" : "")}>
        <p className="text-sm font-sans font-semibold text-text">{title}</p>
        {description ? (
          <p className="max-w-md text-sm font-sans leading-relaxed text-text-muted">
            {description}
          </p>
        ) : null}
      </div>
      {action ? <div className="pt-1">{action}</div> : null}
    </div>
  );
}

export { Empty, emptyVariants };
