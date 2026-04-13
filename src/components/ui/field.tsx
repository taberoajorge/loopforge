import { cva } from "class-variance-authority";
import * as React from "react";

import { cn } from "@/lib/utils";
import { Label } from "./label";

const fieldControlVariants = cva(
  "w-full rounded-md border border-border bg-surface px-3 py-2 font-sans text-sm text-text shadow-sm outline-none transition-[border-color,box-shadow,color,background-color] placeholder:text-text-dim focus-visible:border-primary focus-visible:ring-2 focus-visible:ring-ring/40 focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:border-border/60 disabled:bg-elevated disabled:text-text-dim aria-[invalid=true]:border-destructive aria-[invalid=true]:focus-visible:border-destructive aria-[invalid=true]:focus-visible:ring-destructive/30",
);

const fieldMessageVariants = cva("font-sans text-xs leading-relaxed", {
  variants: {
    tone: {
      default: "text-text-muted",
      danger: "text-destructive",
    },
  },
  defaultVariants: {
    tone: "default",
  },
});

export function getFieldControlClassName(className?: string) {
  return cn(fieldControlVariants(), className);
}

type FieldControlState = {
  disabled?: boolean;
  invalid?: boolean;
  describedBy?: string;
  ariaDescribedBy?: string;
  ariaInvalid?: React.AriaAttributes["aria-invalid"];
};

export function mergeDescribedBy(current: unknown, next?: string) {
  if (typeof current !== "string" || !current.length) return next;
  if (!next) return current;
  return Array.from(new Set(`${current} ${next}`.split(" ").filter(Boolean))).join(" ");
}

export function getFieldControlProps({
  ariaDescribedBy,
  ariaInvalid,
  describedBy,
  disabled,
  invalid = false,
}: FieldControlState) {
  const nextInvalid =
    ariaInvalid === undefined
      ? invalid
      : ariaInvalid === true ||
        ariaInvalid === "true" ||
        ariaInvalid === "grammar" ||
        ariaInvalid === "spelling";

  return {
    disabled,
    "aria-describedby": mergeDescribedBy(ariaDescribedBy, describedBy),
    "aria-invalid": nextInvalid || undefined,
    "data-invalid": nextInvalid || undefined,
  };
}

export type FieldControlProps = React.HTMLAttributes<HTMLDivElement>;

const FieldControl = React.forwardRef<HTMLDivElement, FieldControlProps>(
  ({ className, ...props }, ref) => {
    return <div ref={ref} className={cn("space-y-2", className)} {...props} />;
  },
);

FieldControl.displayName = "FieldControl";

export type FieldMessageProps = React.HTMLAttributes<HTMLParagraphElement> & {
  tone?: "default" | "danger";
};

const FieldMessage = React.forwardRef<HTMLParagraphElement, FieldMessageProps>(
  ({ className, tone = "default", ...props }, ref) => {
    return <p ref={ref} className={cn(fieldMessageVariants({ tone }), className)} {...props} />;
  },
);

FieldMessage.displayName = "FieldMessage";

export type FieldProps = React.HTMLAttributes<HTMLDivElement> & {
  label?: React.ReactNode;
  helperText?: React.ReactNode;
  errorText?: React.ReactNode;
  optionalText?: string;
  required?: boolean;
  invalid?: boolean;
  disabled?: boolean;
  htmlFor?: string;
  children: React.ReactNode;
};

type FieldChildProps = {
  id?: string;
  disabled?: boolean;
  "aria-describedby"?: string;
  "aria-invalid"?: boolean;
  "data-invalid"?: boolean;
};

const Field = React.forwardRef<HTMLDivElement, FieldProps>(
  (
    {
      children,
      className,
      disabled = false,
      errorText,
      helperText,
      htmlFor,
      invalid = false,
      label,
      optionalText,
      required = false,
      ...props
    },
    ref,
  ) => {
    const generatedId = React.useId();
    const child = React.Children.only(children);
    const childElement = React.isValidElement<FieldChildProps>(child) ? child : null;
    const controlId =
      htmlFor ||
      (typeof childElement?.props.id === "string" ? childElement.props.id : undefined) ||
      generatedId;
    const message = errorText ?? helperText;
    const messageId = message ? `${controlId}-message` : undefined;
    const enhancedChild = childElement
      ? React.cloneElement(childElement, {
          id: controlId,
          ...getFieldControlProps({
            describedBy: messageId,
            disabled: childElement.props.disabled ?? disabled,
            invalid,
            ariaDescribedBy: childElement.props["aria-describedby"],
            ariaInvalid: childElement.props["aria-invalid"],
          }),
        })
      : child;

    return (
      <FieldControl ref={ref} className={className} {...props}>
        {label ? (
          <Label
            htmlFor={controlId}
            disabled={disabled}
            invalid={invalid}
            optionalText={optionalText}
            required={required}
          >
            {label}
          </Label>
        ) : null}
        {enhancedChild}
        {message ? (
          <FieldMessage id={messageId} tone={errorText ? "danger" : "default"}>
            {message}
          </FieldMessage>
        ) : null}
      </FieldControl>
    );
  },
);

Field.displayName = "Field";

export { Field, FieldControl, FieldMessage, fieldControlVariants, fieldMessageVariants };
