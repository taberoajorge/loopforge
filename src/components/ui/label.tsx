import { cva, type VariantProps } from "class-variance-authority";
import * as React from "react";

import { cn } from "@/lib/utils";

const labelVariants = cva(
  "flex items-center gap-2 font-medium font-sans text-text-muted text-xs uppercase tracking-[0.16em]",
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
  ({ children, className, disabled, htmlFor, invalid, optionalText, required, ...props }, ref) => {
    const content = (
      <>
        <span className="min-w-0">{children}</span>
        {required ? <span className="text-primary">*</span> : null}
        {optionalText ? (
          <span className="font-mono text-[10px] text-text-dim tracking-[0.24em]">
            {optionalText}
          </span>
        ) : null}
      </>
    );

    if (htmlFor) {
      return (
        <label
          ref={ref}
          htmlFor={htmlFor}
          className={cn(labelVariants({ disabled, invalid }), className)}
          {...props}
        >
          {content}
        </label>
      );
    }

    return (
      <div
        ref={ref as React.Ref<HTMLDivElement>}
        className={cn(labelVariants({ disabled, invalid }), className)}
      >
        {content}
      </div>
    );
  },
);

Label.displayName = "Label";

export { Label, labelVariants };
