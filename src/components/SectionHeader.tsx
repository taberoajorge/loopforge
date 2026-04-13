import type * as React from "react";
import { CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Separator, type SeparatorProps } from "@/components/ui/separator";
import { cn } from "@/lib/utils";

export type SectionHeaderProps = Omit<React.HTMLAttributes<HTMLDivElement>, "title"> & {
  title: React.ReactNode;
  description?: React.ReactNode;
  eyebrow?: React.ReactNode;
  actions?: React.ReactNode;
  compact?: boolean;
  showSeparator?: boolean;
  separatorTone?: SeparatorProps["tone"];
};

export function SectionHeader({
  actions,
  className,
  compact = false,
  description,
  eyebrow,
  separatorTone = "default",
  showSeparator = false,
  title,
  ...props
}: SectionHeaderProps) {
  return (
    <div className={cn("space-y-3", className)} {...props}>
      <CardHeader className={cn("border-none p-0", compact ? "gap-1" : "gap-1.5")}>
        {eyebrow ? (
          <p className="font-sans text-text-muted text-xs uppercase tracking-wider">{eyebrow}</p>
        ) : null}
        <div className="flex items-start justify-between gap-3">
          <CardTitle className="text-base">{title}</CardTitle>
          {actions ? <div className="shrink-0">{actions}</div> : null}
        </div>
        {description ? <CardDescription className="text-sm">{description}</CardDescription> : null}
      </CardHeader>
      {showSeparator ? <Separator tone={separatorTone} /> : null}
    </div>
  );
}
