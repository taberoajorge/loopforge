import type { ReactNode } from "react";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "../../../components/ui/resizable";

type PlanWorkspaceProps = {
  streamPanel: ReactNode;
  previewPanel: ReactNode;
};

export function PlanWorkspace({ streamPanel, previewPanel }: PlanWorkspaceProps) {
  return (
    <ResizablePanelGroup direction="horizontal" className="min-h-0 flex-1">
      <ResizablePanel defaultSize={55} minSize={30} maxSize={70} className="min-h-0">
        {streamPanel}
      </ResizablePanel>
      <ResizableHandle />
      <ResizablePanel defaultSize={45} minSize={30} maxSize={70} className="min-h-0">
        {previewPanel}
      </ResizablePanel>
    </ResizablePanelGroup>
  );
}
