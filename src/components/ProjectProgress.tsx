import * as React from "react";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  type CardProps,
} from "@/components/ui/card";
import { Progress, type ProgressProps } from "@/components/ui/progress";
import {
  StatusBadge,
  type StatusBadgeProps,
  type StatusBadgeStatus,
} from "@/components/StatusBadge";
import { cn } from "@/lib/utils";

export type ProjectProgressProps = Omit<CardProps, "title"> & {
  title: React.ReactNode;
  value: number;
  total?: number;
  hint?: React.ReactNode;
  status?: StatusBadgeStatus;
  statusLabel?: React.ReactNode;
  progressTone?: ProgressProps["tone"];
  indeterminate?: boolean;
  size?: ProgressProps["size"];
  showValue?: boolean;
  badgeEmphasis?: StatusBadgeProps["emphasis"];
};

export function ProjectProgress({
  badgeEmphasis = "subtle",
  className,
  hint,
  indeterminate = false,
  progressTone = "default",
  showValue = true,
  size = "md",
  status,
  statusLabel,
  title,
  total = 100,
  value,
  variant = "surface",
  ...props
}: ProjectProgressProps) {
  return (
    <Card variant={variant} className={className} {...props}>
      <CardHeader className="flex-row items-center justify-between gap-3">
        <CardTitle className="text-sm">{title}</CardTitle>
        {status ? (
          <StatusBadge status={status} label={statusLabel} emphasis={badgeEmphasis} />
        ) : null}
      </CardHeader>
      <CardContent className={cn("pt-3")}>
        <Progress
          value={value}
          max={total}
          hint={hint}
          tone={progressTone}
          indeterminate={indeterminate}
          showValue={showValue}
          size={size}
          valueLabel={indeterminate ? "..." : `${value}/${total}`}
        />
      </CardContent>
    </Card>
  );
}
