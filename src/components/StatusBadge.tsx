import type * as React from "react";
import { Badge, type BadgeProps } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import { useDisplayVocabularyStore } from "@/stores/displayVocabularyStore";

export type StatusBadgeStatus =
  | "draft"
  | "pending"
  | "current"
  | "running"
  | "paused"
  | "blocked"
  | "completed"
  | "success"
  | "error"
  | "failed"
  | "archived";

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
  const vocabulary = useDisplayVocabularyStore((state) => state.vocabulary);
  const variant =
    vocabulary?.statusVariants[status] ??
    (status === "paused"
      ? "warning"
      : status === "blocked" || status === "error" || status === "failed"
        ? "danger"
        : status === "completed" || status === "success"
          ? "success"
          : status === "running" || status === "current"
            ? "info"
            : "neutral");
  const fallbackLabel = `${status.slice(0, 1).toUpperCase()}${status.slice(1)}`;
  const content = label ?? vocabulary?.statusLabels[status] ?? fallbackLabel;

  return (
    <Badge
      variant={variant as NonNullable<BadgeProps["variant"]>}
      emphasis={emphasis}
      className={cn(uppercase ? "uppercase tracking-wide" : "", className)}
      {...props}
    >
      {leading}
      {content}
      {trailing}
    </Badge>
  );
}
