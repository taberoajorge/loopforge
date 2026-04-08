import type { ReactNode } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../../../components/ui/card";

type TerminalFrameProps = {
  children: ReactNode;
  controls?: ReactNode;
  subtitle?: string;
  title?: string;
};

export function TerminalFrame({
  children,
  controls,
  subtitle = "Live agent stream",
  title = "Raw Output",
}: TerminalFrameProps) {
  return (
    <div className="h-full bg-void p-4">
      <Card className="flex h-full min-h-0 flex-col overflow-hidden">
        <CardHeader className="flex-row items-center justify-between gap-3 px-4 py-3">
          <div className="min-w-0">
            <CardTitle className="truncate text-xs font-mono uppercase tracking-wider">{title}</CardTitle>
            <CardDescription className="mt-1">{subtitle}</CardDescription>
          </div>
          {controls ? <div className="flex shrink-0 items-center gap-2">{controls}</div> : null}
        </CardHeader>
        <CardContent className="min-h-0 flex-1 p-3">{children}</CardContent>
      </Card>
    </div>
  );
}
