import { cva, type VariantProps } from "class-variance-authority";
import * as React from "react";

import { cn } from "@/lib/utils";

export type ScrollAreaProps = React.HTMLAttributes<HTMLDivElement>;

const ScrollArea = React.forwardRef<HTMLDivElement, ScrollAreaProps>(
  ({ className, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn("relative min-h-0 min-w-0 overflow-hidden", className)}
        {...props}
      />
    );
  },
);

ScrollArea.displayName = "ScrollArea";

const scrollViewportVariants = cva("h-full min-h-0 w-full min-w-0 overscroll-contain", {
  variants: {
    orientation: {
      both: "overflow-auto",
      horizontal: "overflow-x-auto overflow-y-hidden",
      vertical: "overflow-y-auto overflow-x-hidden",
    },
    padding: {
      none: "",
      sm: "p-2",
      md: "p-3",
    },
  },
  defaultVariants: {
    orientation: "vertical",
    padding: "none",
  },
});

export type ScrollViewportProps = React.HTMLAttributes<HTMLDivElement> &
  VariantProps<typeof scrollViewportVariants>;

const ScrollViewport = React.forwardRef<HTMLDivElement, ScrollViewportProps>(
  ({ className, orientation, padding, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(scrollViewportVariants({ orientation, padding }), className)}
        data-orientation={orientation}
        {...props}
      />
    );
  },
);

ScrollViewport.displayName = "ScrollViewport";

export type ScrollContentProps = React.HTMLAttributes<HTMLDivElement>;

const ScrollContent = React.forwardRef<HTMLDivElement, ScrollContentProps>(
  ({ className, ...props }, ref) => {
    return <div ref={ref} className={cn("min-w-full", className)} {...props} />;
  },
);

ScrollContent.displayName = "ScrollContent";

export { ScrollArea, ScrollContent, ScrollViewport, scrollViewportVariants };
