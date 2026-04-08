import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";

const labelVariants = cva(
  "flex items-center gap-2 text-xs font-sans font-medium uppercase tracking-[0.16em] text-text-muted",
  {
    variants: {
      disabled: {
        true: "text-text-dim",
        false: "",
      },
      invalid: {
        true: "text-destructive",
        false: "",
      },
    },
    defaultVariants: {
      disabled: false,
      invalid: false,
    },
  },
);

export type LabelProps = React.LabelHTMLAttributes<HTMLLabelElement> &
  VariantProps<typeof labelVariants> & {
    optionalText?: string;
    required?: boolean;
  };

const Label = React.forwardRef<HTMLLabelElement, LabelProps>(
  ({ children, className, disabled, invalid, optionalText, required, ...props }, ref) => {
    return (
      <label
        ref={ref}
        className={cn(labelVariants({ disabled, invalid }), className)}
        {...props}
      >
        <span className="min-w-0">{children}</span>
        {required ? <span className="text-primary">*</span> : null}
        {optionalText ? (
          <span className="text-[10px] font-mono tracking-[0.24em] text-text-dim">
            {optionalText}
          </span>
        ) : null}
      </label>
    );
  },
);

Label.displayName = "Label";

export { Label, labelVariants };
