import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useMemo, useState } from "react";
import { isApplePlatform } from "../lib/platform";
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
  const isMac = useMemo(() => isApplePlatform(), []);

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

  return (
    <div
      data-tauri-drag-region
      className="flex h-9 shrink-0 select-none items-center justify-between border-border border-b bg-void"
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
          className="font-mono text-[10px] text-text-dim uppercase tracking-[0.25em]"
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
