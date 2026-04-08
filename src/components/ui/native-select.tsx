import * as React from "react";

import { cn } from "@/lib/utils";
import { getFieldControlClassName, getFieldControlProps } from "./field";

export type NativeSelectProps = React.SelectHTMLAttributes<HTMLSelectElement> & {
  invalid?: boolean;
  placeholder?: string;
};

const NativeSelect = React.forwardRef<HTMLSelectElement, NativeSelectProps>(
  ({ children, className, defaultValue, invalid = false, placeholder, value, ...props }, ref) => {
    const hasSelection =
      value !== undefined
        ? `${value}`.length > 0
        : defaultValue !== undefined
          ? `${defaultValue}`.length > 0
          : false;

    return (
      <div className="relative">
        <select
          ref={ref}
          defaultValue={defaultValue}
          value={value}
          {...getFieldControlProps({
            invalid,
            disabled: props.disabled,
            ariaDescribedBy: props["aria-describedby"],
            ariaInvalid: props["aria-invalid"],
          })}
          className={cn(
            getFieldControlClassName(),
            "h-10 appearance-none pr-10",
            !hasSelection && placeholder ? "text-text-dim" : "",
            className,
          )}
          {...props}
        >
          {placeholder ? (
            <option value="" disabled>
              {placeholder}
            </option>
          ) : null}
          {children}
        </select>
        <span className="pointer-events-none absolute inset-y-0 right-3 flex items-center text-text-dim">
          <svg
            aria-hidden="true"
            className="h-4 w-4"
            viewBox="0 0 16 16"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              d="M4 6L8 10L12 6"
              stroke="currentColor"
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="1.5"
            />
          </svg>
        </span>
      </div>
    );
  },
);

NativeSelect.displayName = "NativeSelect";

export { NativeSelect };
