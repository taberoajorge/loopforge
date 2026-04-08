import * as React from "react";
import { Badge, type BadgeProps } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

const STATUS_LABELS = {
  draft: "Draft",
  pending: "Pending",
  current: "Current",
  running: "Running",
  paused: "Paused",
  blocked: "Blocked",
  completed: "Completed",
  success: "Success",
  error: "Error",
  failed: "Failed",
  archived: "Archived",
} as const;

const STATUS_VARIANTS: Record<StatusBadgeStatus, NonNullable<BadgeProps["variant"]>> = {
  draft: "neutral",
  pending: "neutral",
  current: "info",
  running: "info",
  paused: "warning",
  blocked: "danger",
  completed: "success",
  success: "success",
  error: "danger",
  failed: "danger",
  archived: "neutral",
};

export type StatusBadgeStatus = keyof typeof STATUS_LABELS;

export type StatusBadgeProps = Omit<BadgeProps, "children" | "variant"> & {
  status: StatusBadgeStatus;
  label?: React.ReactNode;
  leading?: React.ReactNode;
  trailing?: React.ReactNode;
  uppercase?: boolean;
};

export function StatusBadge({
  className,
  emphasis = "subtle",
  label,
  leading,
  status,
  trailing,
  uppercase = false,
  ...props
}: StatusBadgeProps) {
  const content = label ?? STATUS_LABELS[status];

  return (
    <Badge
      variant={STATUS_VARIANTS[status]}
      emphasis={emphasis}
      className={cn(uppercase ? "tracking-wide uppercase" : "", className)}
      {...props}
    >
      {leading}
      {content}
      {trailing}
    </Badge>
  );
}
