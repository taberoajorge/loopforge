import { useState, useEffect, useMemo, type MouseEvent } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { WindowControls } from "./title-bar/WindowControls";
import { Button } from "./ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "./ui/dropdown-menu";

export function TitleBar() {
  const appWindow = getCurrentWindow();
  const [isMaximized, setIsMaximized] = useState(false);
  const isMac = useMemo(
    () =>
      typeof navigator !== "undefined" &&
      /Mac|iPhone|iPad|iPod/.test(navigator.userAgent),
    [],
  );

  useEffect(() => {
    void appWindow.isMaximized().then(setIsMaximized);
  }, [appWindow]);

  async function handleToggleMaximize() {
    await appWindow.toggleMaximize();
    setIsMaximized(await appWindow.isMaximized());
  }

  function handleMinimize() {
    void appWindow.minimize();
  }

  function handleClose() {
    void appWindow.close();
  }

  function handleDragStart(event: MouseEvent<HTMLDivElement>) {
    if (event.button !== 0) {
      return;
    }
    const target = event.target as HTMLElement;
    if (target.closest("[data-no-drag]")) {
      return;
    }
    void appWindow.startDragging();
  }

  return (
    <div
      data-tauri-drag-region
      onMouseDown={handleDragStart}
      className="h-9 flex items-center justify-between bg-void border-b border-border select-none shrink-0"
    >
      <div className="flex items-center gap-2 px-2" data-tauri-drag-region>
        {isMac ? (
          <WindowControls
            platform="mac"
            isMaximized={isMaximized}
            onMinimize={handleMinimize}
            onToggleMaximize={() => void handleToggleMaximize()}
            onClose={handleClose}
          />
        ) : null}
        <span
          data-tauri-drag-region
          className="text-text-dim text-[10px] font-mono uppercase tracking-[0.25em]"
        >
          LoopForge
        </span>
      </div>
      <div className="flex items-center gap-1 px-2" data-no-drag>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              className="h-7 w-7 text-text-muted hover:bg-elevated"
              data-no-drag
              aria-label="Open window menu"
            >
              <span className="text-sm leading-none">⋯</span>
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onSelect={handleMinimize}>Minimize</DropdownMenuItem>
            <DropdownMenuItem onSelect={() => void handleToggleMaximize()}>
              {isMaximized ? "Restore" : "Maximize"}
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem onSelect={handleClose}>Close</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
        {isMac ? null : (
          <WindowControls
            platform="default"
            isMaximized={isMaximized}
            onMinimize={handleMinimize}
            onToggleMaximize={() => void handleToggleMaximize()}
            onClose={handleClose}
          />
        )}
      </div>
    </div>
  );
}
