import type * as React from "react";
import { Badge } from "@/components/ui/badge";
import { Switch, type SwitchProps } from "@/components/ui/switch";
import { cn } from "@/lib/utils";

export type ThemeMode = "light" | "dark";

export type ThemeToggleProps = Omit<React.HTMLAttributes<HTMLDivElement>, "onChange"> & {
  value: ThemeMode;
  onValueChange: (value: ThemeMode) => void;
  disabled?: boolean;
  label?: React.ReactNode;
  description?: React.ReactNode;
  showModeBadge?: boolean;
  size?: SwitchProps["size"];
};

export function ThemeToggle({
  className,
  description,
  disabled = false,
  label = "Theme",
  onValueChange,
  showModeBadge = true,
  size = "md",
  value,
  ...props
}: ThemeToggleProps) {
  const isDarkMode = value === "dark";
  const modeLabel = isDarkMode ? "Dark" : "Light";

  return (
    <div className={cn("flex items-center justify-between gap-3", className)} {...props}>
      <div className="min-w-0">
        <p className="ui-type-title font-medium font-sans text-text">{label}</p>
        {description ? (
          <p className="ui-type-body font-sans text-text-muted">{description}</p>
        ) : null}
      </div>
      <div className="flex items-center gap-2">
        {showModeBadge ? (
          <Badge variant={isDarkMode ? "info" : "neutral"} className="min-w-12 justify-center">
            {modeLabel}
          </Badge>
        ) : null}
        <Switch
          checked={isDarkMode}
          disabled={disabled}
          size={size}
          onCheckedChange={(checked) => onValueChange(checked ? "dark" : "light")}
        />
      </div>
    </div>
  );
}
