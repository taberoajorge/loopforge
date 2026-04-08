import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";

const spinnerVariants = cva("inline-block animate-spin rounded-full border-2 border-current border-r-transparent", {
  variants: {
    size: {
      sm: "h-4 w-4",
      md: "h-6 w-6",
      lg: "h-8 w-8",
    },
    tone: {
      default: "text-text-muted",
      primary: "text-primary",
      danger: "text-destructive",
    },
  },
  defaultVariants: {
    size: "md",
    tone: "default",
  },
});

export type SpinnerProps = React.HTMLAttributes<HTMLSpanElement> &
  VariantProps<typeof spinnerVariants> & {
    label?: string;
    message?: React.ReactNode;
    fullWidth?: boolean;
  };

function Spinner({
  className,
  fullWidth = false,
  label = "Loading",
  message,
  size,
  tone,
  ...props
}: SpinnerProps) {
  return (
    <span
      role="status"
      aria-label={label}
      className={cn(
        "inline-flex items-center justify-center gap-2",
        fullWidth ? "w-full" : "",
        className,
      )}
      {...props}
    >
      <span aria-hidden="true" className={spinnerVariants({ size, tone })} />
      {message ? <span className="text-sm font-sans text-text-muted">{message}</span> : null}
    </span>
  );
}

export { Spinner, spinnerVariants };
