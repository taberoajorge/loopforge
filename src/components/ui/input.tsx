import * as React from "react";

import { cn } from "@/lib/utils";
import { getFieldControlClassName, getFieldControlProps } from "./field";

export type InputProps = React.InputHTMLAttributes<HTMLInputElement> & {
  invalid?: boolean;
};

const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, invalid = false, type = "text", ...props }, ref) => {
    return (
      <input
        ref={ref}
        type={type}
        {...getFieldControlProps({
          invalid,
          disabled: props.disabled,
          ariaDescribedBy: props["aria-describedby"],
          ariaInvalid: props["aria-invalid"],
        })}
        className={cn(
          getFieldControlClassName(),
          "h-10 min-w-0",
          type === "file" ? "p-1 text-xs" : "",
          className,
        )}
        {...props}
      />
    );
  },
);

Input.displayName = "Input";

export { Input };
