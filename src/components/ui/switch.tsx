import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";
import { getFieldControlProps } from "./field";

const switchVariants = cva(
  "group inline-flex shrink-0 items-center rounded-full border border-border bg-elevated transition-[background-color,border-color,box-shadow] outline-none focus-visible:ring-2 focus-visible:ring-ring/40 focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-60 aria-[invalid=true]:border-destructive data-[state=checked]:bg-primary data-[state=checked]:border-primary",
  {
    variants: {
      size: {
        sm: "h-5 w-9 p-0.5",
        md: "h-6 w-11 p-0.5",
      },
    },
    defaultVariants: {
      size: "md",
    },
  },
);

const thumbVariants = cva(
  "rounded-full bg-text shadow-sm transition-transform data-[state=checked]:translate-x-full group-data-[state=checked]:bg-primary-foreground",
  {
    variants: {
      size: {
        sm: "h-4 w-4",
        md: "h-5 w-5",
      },
    },
    defaultVariants: {
      size: "md",
    },
  },
);

export type SwitchProps = Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "onChange"> &
  VariantProps<typeof switchVariants> & {
    checked?: boolean;
    defaultChecked?: boolean;
    invalid?: boolean;
    onCheckedChange?: (checked: boolean) => void;
  };

const Switch = React.forwardRef<HTMLButtonElement, SwitchProps>(
  (
    {
      checked,
      className,
      defaultChecked = false,
      disabled,
      invalid = false,
      onCheckedChange,
      onClick,
      size,
      type,
      ...props
    },
    ref,
  ) => {
    const [internalChecked, setInternalChecked] = React.useState(defaultChecked);
    const isControlled = checked !== undefined;
    const isChecked = isControlled ? checked : internalChecked;

    return (
      <button
        ref={ref}
        type={type ?? "button"}
        role="switch"
        aria-checked={isChecked}
        {...getFieldControlProps({
          invalid,
          disabled,
          ariaDescribedBy: props["aria-describedby"],
          ariaInvalid: props["aria-invalid"],
        })}
        data-state={isChecked ? "checked" : "unchecked"}
        className={cn(switchVariants({ size }), className)}
        disabled={disabled}
        onClick={(event) => {
          onClick?.(event);
          if (event.defaultPrevented || disabled) return;
          const nextChecked = !isChecked;
          if (!isControlled) setInternalChecked(nextChecked);
          onCheckedChange?.(nextChecked);
        }}
        {...props}
      >
        <span
          aria-hidden="true"
          data-state={isChecked ? "checked" : "unchecked"}
          className={thumbVariants({ size })}
        />
      </button>
    );
  },
);

Switch.displayName = "Switch";

export { Switch, switchVariants };
