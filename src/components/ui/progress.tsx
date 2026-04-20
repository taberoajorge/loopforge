import { cva, type VariantProps } from "class-variance-authority";
import type * as React from "react";

import { cn } from "@/lib/utils";

const progressIndicatorVariants = cva(
  "h-full rounded-full transition-[width,background-color] duration-300",
  {
    variants: {
      tone: {
        default: "bg-primary",
        info: "bg-running",
        danger: "bg-destructive",
      },
    },
    defaultVariants: {
      tone: "default",
    },
  },
);

export type ProgressProps = React.HTMLAttributes<HTMLDivElement> &
  VariantProps<typeof progressIndicatorVariants> & {
    value?: number;
    max?: number;
    label?: React.ReactNode;
    hint?: React.ReactNode;
    valueLabel?: React.ReactNode;
    showValue?: boolean;
    indeterminate?: boolean;
    size?: "sm" | "md";
  };

function clampPercentage(value: number, max: number) {
  if (max <= 0) return 0;
  return Math.min(100, Math.max(0, (value / max) * 100));
}

function asTitle(value: React.ReactNode): string | undefined {
  if (typeof value === "string") return value;
  if (typeof value === "number") return String(value);
  return undefined;
}

function Progress({
  className,
  hint,
  indeterminate = false,
  label,
  max = 100,
  showValue = true,
  size = "md",
  tone,
  value = 0,
  valueLabel,
  ...props
}: ProgressProps) {
  const percentage = clampPercentage(value, max);

  return (
    <div className={cn("space-y-2", className)} {...props}>
      {label || hint || showValue ? (
        <div className="flex items-center justify-between gap-3 font-sans text-xs">
          <div className="min-w-0">
            {label ? (
              <p className="truncate font-medium text-text" title={asTitle(label)}>
                {label}
              </p>
            ) : null}
            {hint ? (
              <p className="truncate text-text-muted" title={asTitle(hint)}>
                {hint}
              </p>
            ) : null}
          </div>
          {showValue ? (
            <span className="shrink-0 font-mono text-text-muted">
              {valueLabel ?? (indeterminate ? "..." : `${Math.round(percentage)}%`)}
            </span>
          ) : null}
        </div>
      ) : null}
      <div
        role="progressbar"
        aria-valuemax={max}
        aria-valuemin={0}
        aria-valuenow={indeterminate ? undefined : value}
        className={cn(
          "overflow-hidden rounded-full border border-border/60 bg-elevated",
          size === "sm" ? "h-1.5" : "h-2",
        )}
      >
        <div
          className={cn(
            progressIndicatorVariants({ tone }),
            indeterminate ? "w-2/5 animate-pulse" : "",
          )}
          style={{ width: indeterminate ? undefined : `${percentage}%` }}
        />
      </div>
    </div>
  );
}

export { Progress, progressIndicatorVariants };
