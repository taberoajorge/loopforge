import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";

const separatorVariants = cva("shrink-0", {
  variants: {
    tone: {
      muted: "bg-border/60",
      default: "bg-border",
      strong: "bg-text-dim",
    },
  },
  defaultVariants: {
    tone: "default",
  },
});

export type SeparatorProps = React.HTMLAttributes<HTMLDivElement> &
  VariantProps<typeof separatorVariants> & {
    orientation?: "horizontal" | "vertical";
    decorative?: boolean;
  };

const Separator = React.forwardRef<HTMLDivElement, SeparatorProps>(
  (
    {
      className,
      orientation = "horizontal",
      decorative = true,
      tone,
      ...props
    },
    ref,
  ) => {
    return (
      <div
        ref={ref}
        role={decorative ? "presentation" : "separator"}
        aria-hidden={decorative}
        data-orientation={orientation}
        className={cn(
          separatorVariants({ tone }),
          orientation === "horizontal" ? "h-px w-full" : "h-full w-px",
          className,
        )}
        {...props}
      />
    );
  },
);
Separator.displayName = "Separator";

export { Separator, separatorVariants };
