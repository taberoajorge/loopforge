import * as React from "react";

import { cn } from "@/lib/utils";
import { getFieldControlClassName, getFieldControlProps } from "./field";

export type TextareaProps = React.TextareaHTMLAttributes<HTMLTextAreaElement> & {
  invalid?: boolean;
};

const Textarea = React.forwardRef<HTMLTextAreaElement, TextareaProps>(
  ({ className, invalid = false, rows = 5, ...props }, ref) => {
    return (
      <textarea
        ref={ref}
        rows={rows}
        {...getFieldControlProps({
          invalid,
          disabled: props.disabled,
          ariaDescribedBy: props["aria-describedby"],
          ariaInvalid: props["aria-invalid"],
        })}
        className={cn(
          getFieldControlClassName(),
          "min-h-28 resize-y leading-relaxed",
          className,
        )}
        {...props}
      />
    );
  },
);

Textarea.displayName = "Textarea";

export { Textarea };
