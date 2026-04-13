import { Button } from "../ui/button";

type WindowControlsProps = {
  isMaximized: boolean;
  onClose: () => void;
  onMinimize: () => void;
  onToggleMaximize: () => void;
  platform: "mac" | "default";
};

export function WindowControls({
  isMaximized,
  onClose,
  onMinimize,
  onToggleMaximize,
  platform,
}: WindowControlsProps) {
  if (platform === "mac") {
    return (
      <div className="flex h-full items-center gap-2 px-2" data-no-drag>
        <button
          type="button"
          className="h-3 w-3 rounded-full bg-blocked transition-opacity hover:opacity-85"
          onClick={onClose}
          aria-label="Close window"
          data-no-drag
        />
        <button
          type="button"
          className="h-3 w-3 rounded-full bg-paused transition-opacity hover:opacity-85"
          onClick={onMinimize}
          aria-label="Minimize window"
          data-no-drag
        />
        <button
          type="button"
          className="h-3 w-3 rounded-full bg-success transition-opacity hover:opacity-85"
          onClick={onToggleMaximize}
          aria-label={isMaximized ? "Restore window" : "Maximize window"}
          data-no-drag
        />
      </div>
    );
  }

  return (
    <div className="flex h-full items-center" data-no-drag>
      <Button
        variant="ghost"
        className="h-full w-11 rounded-none text-text-muted hover:bg-elevated"
        onClick={onMinimize}
        aria-label="Minimize window"
        data-no-drag
      >
        <svg width="10" height="1" viewBox="0 0 10 1">
          <title>Minimize</title>
          <rect fill="currentColor" width="10" height="1" />
        </svg>
      </Button>
      <Button
        variant="ghost"
        className="h-full w-11 rounded-none text-text-muted hover:bg-elevated"
        onClick={onToggleMaximize}
        aria-label={isMaximized ? "Restore window" : "Maximize window"}
        data-no-drag
      >
        {isMaximized ? (
          <svg width="10" height="10" viewBox="0 0 10 10">
            <title>Restore</title>
            <path fill="none" stroke="currentColor" strokeWidth="1" d="M3 1h6v6M1 3h6v6" />
          </svg>
        ) : (
          <svg width="10" height="10" viewBox="0 0 10 10">
            <title>Maximize</title>
            <rect
              fill="none"
              stroke="currentColor"
              strokeWidth="1"
              x="0.5"
              y="0.5"
              width="9"
              height="9"
            />
          </svg>
        )}
      </Button>
      <Button
        variant="ghost"
        className="h-full w-11 rounded-none text-text-muted hover:bg-blocked/80 hover:text-text"
        onClick={onClose}
        aria-label="Close window"
        data-no-drag
      >
        <svg width="10" height="10" viewBox="0 0 10 10">
          <title>Close</title>
          <path stroke="currentColor" strokeWidth="1.2" d="M1 1l8 8M9 1l-8 8" />
        </svg>
      </Button>
    </div>
  );
}
