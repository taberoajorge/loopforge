import * as React from "react";
import { Button, type ButtonProps } from "@/components/ui/button";
import { Empty, type EmptyProps } from "@/components/ui/empty";

export type EmptyStateAction = {
  label: React.ReactNode;
  onClick?: () => void;
  disabled?: boolean;
  variant?: ButtonProps["variant"];
};

export type EmptyStateProps = Omit<EmptyProps, "action"> & {
  primaryAction?: EmptyStateAction;
  secondaryAction?: EmptyStateAction;
  action?: React.ReactNode;
};

export function EmptyState({
  action,
  primaryAction,
  secondaryAction,
  ...props
}: EmptyStateProps) {
  let actionContent = action;

  if (!actionContent && (primaryAction || secondaryAction)) {
    actionContent = (
      <div className="flex items-center justify-center gap-2">
        {secondaryAction ? (
          <Button
            variant={secondaryAction.variant ?? "ghost"}
            size="sm"
            disabled={secondaryAction.disabled}
            onClick={secondaryAction.onClick}
          >
            {secondaryAction.label}
          </Button>
        ) : null}
        {primaryAction ? (
          <Button
            variant={primaryAction.variant ?? "primary"}
            size="sm"
            disabled={primaryAction.disabled}
            onClick={primaryAction.onClick}
          >
            {primaryAction.label}
          </Button>
        ) : null}
      </div>
    );
  }

  return <Empty action={actionContent} {...props} />;
}
