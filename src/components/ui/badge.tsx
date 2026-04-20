import { cva, type VariantProps } from "class-variance-authority";
import type * as React from "react";

import { cn } from "@/lib/utils";

const badgeVariants = cva(
  "inline-flex items-center rounded-full border px-2 py-0.5 font-sans font-semibold text-[11px] leading-none transition-colors",
  {
    variants: {
      variant: {
        neutral: "border-border bg-elevated text-text-muted",
        info: "border-cyan/30 bg-cyan/10 text-cyan",
        success: "border-success/30 bg-success/10 text-success",
        warning: "border-paused/30 bg-paused/10 text-paused",
        danger: "border-blocked/30 bg-blocked/10 text-blocked",
      },
      emphasis: {
        subtle: "",
        solid: "border-transparent",
      },
    },
    compoundVariants: [
      {
        variant: "info",
        emphasis: "solid",
        className: "bg-cyan text-void",
      },
      {
        variant: "success",
        emphasis: "solid",
        className: "bg-success text-void",
      },
      {
        variant: "warning",
        emphasis: "solid",
        className: "bg-paused text-void",
      },
      {
        variant: "danger",
        emphasis: "solid",
        className: "bg-blocked text-void",
      },
      {
        variant: "neutral",
        emphasis: "solid",
        className: "bg-text text-void",
      },
    ],
    defaultVariants: {
      variant: "neutral",
      emphasis: "subtle",
    },
  },
);

export type BadgeProps = React.HTMLAttributes<HTMLSpanElement> & VariantProps<typeof badgeVariants>;

function Badge({ className, variant, emphasis, ...props }: BadgeProps) {
  return <span className={cn(badgeVariants({ variant, emphasis }), className)} {...props} />;
}

export { Badge, badgeVariants };
