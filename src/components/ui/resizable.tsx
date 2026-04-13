import * as React from "react";

import { cn } from "@/lib/utils";

type PanelConfig = { defaultSize?: number; maxSize?: number; minSize?: number };
type Direction = "horizontal" | "vertical";
type ResizableContextValue = {
  containerRef: React.RefObject<HTMLDivElement | null>;
  direction: Direction;
  panels: PanelConfig[];
  setSizes: React.Dispatch<React.SetStateAction<number[]>>;
  sizes: number[];
};

const ResizableContext = React.createContext<ResizableContextValue | null>(null);

function useResizableContext(component: string) {
  const context = React.useContext(ResizableContext);
  if (!context) throw new Error(`${component} must be used within ResizablePanelGroup`);
  return context;
}

function buildSizes(panels: PanelConfig[]) {
  if (!panels.length) return [];
  const fallback = 100 / panels.length;
  const baseSizes = panels.map((panel) => panel.defaultSize ?? fallback);
  const total = baseSizes.reduce((sum, size) => sum + size, 0) || 100;
  return baseSizes.map((size) => (size / total) * 100);
}

function assignRef<T>(ref: React.ForwardedRef<T>, value: T) {
  if (typeof ref === "function") return ref(value);
  if (ref) ref.current = value;
}

function resizePanels(sizes: number[], panels: PanelConfig[], handleIndex: number, delta: number) {
  const left = sizes[handleIndex] ?? 0;
  const right = sizes[handleIndex + 1] ?? 0;
  if (!left || !right) return sizes;
  const leftPanel = panels[handleIndex];
  const rightPanel = panels[handleIndex + 1];
  const minDelta = Math.max((leftPanel?.minSize ?? 15) - left, right - (rightPanel?.maxSize ?? 85));
  const maxDelta = Math.min((leftPanel?.maxSize ?? 85) - left, right - (rightPanel?.minSize ?? 15));
  const safeDelta = Math.max(minDelta, Math.min(maxDelta, delta));
  return sizes.map((size, index) => {
    if (index === handleIndex) return size + safeDelta;
    if (index === handleIndex + 1) return size - safeDelta;
    return size;
  });
}

export type ResizablePanelGroupProps = React.HTMLAttributes<HTMLDivElement> & {
  direction?: Direction;
};
export type ResizablePanelProps = React.HTMLAttributes<HTMLDivElement> & PanelConfig;
export type ResizableHandleProps = React.ButtonHTMLAttributes<HTMLButtonElement> & {
  step?: number;
  withGrip?: boolean;
};

type InternalPanelProps = ResizablePanelProps & { panelIndex?: number };
type InternalHandleProps = ResizableHandleProps & { handleIndex?: number };

const ResizablePanel = React.forwardRef<HTMLDivElement, InternalPanelProps>(
  ({ className, panelIndex = 0, style, ...props }, ref) => {
    const { sizes } = useResizableContext("ResizablePanel");
    const size = sizes[panelIndex] ?? 100;
    return (
      <div
        ref={ref}
        className={cn("min-h-0 min-w-0 overflow-hidden", className)}
        style={{ ...style, flexBasis: `${size}%`, flexGrow: 0, flexShrink: 0 }}
        {...props}
      />
    );
  },
);

ResizablePanel.displayName = "ResizablePanel";

const ResizableHandle = React.forwardRef<HTMLButtonElement, InternalHandleProps>(
  (
    { className, handleIndex = 0, onKeyDown, onPointerDown, step = 5, withGrip = true, ...props },
    ref,
  ) => {
    const { containerRef, direction, panels, setSizes, sizes } =
      useResizableContext("ResizableHandle");
    const startRef = React.useRef<{ point: number; sizes: number[] } | null>(null);

    React.useEffect(() => {
      function handlePointerMove(event: PointerEvent) {
        const start = startRef.current;
        const container = containerRef.current;
        if (!start || !container) return;
        const rect = container.getBoundingClientRect();
        const total = direction === "horizontal" ? rect.width : rect.height;
        const point = direction === "horizontal" ? event.clientX : event.clientY;
        if (!total) return;
        const delta = ((point - start.point) / total) * 100;
        setSizes(resizePanels(start.sizes, panels, handleIndex, delta));
      }

      function handlePointerUp() {
        startRef.current = null;
        document.body.style.cursor = "";
        document.body.style.userSelect = "";
      }

      window.addEventListener("pointermove", handlePointerMove);
      window.addEventListener("pointerup", handlePointerUp);
      return () => {
        window.removeEventListener("pointermove", handlePointerMove);
        window.removeEventListener("pointerup", handlePointerUp);
      };
    }, [containerRef, direction, handleIndex, panels, setSizes]);

    return (
      <button
        type="button"
        ref={ref}
        aria-label={direction === "horizontal" ? "Resize columns" : "Resize rows"}
        className={cn(
          "group relative shrink-0 bg-border/50 transition-colors hover:bg-primary/30",
          direction === "horizontal" ? "w-1.5 cursor-col-resize" : "h-1.5 cursor-row-resize",
          className,
        )}
        onKeyDown={(event) => {
          onKeyDown?.(event);
          if (event.defaultPrevented) return;
          const negativeKey = direction === "horizontal" ? "ArrowLeft" : "ArrowUp";
          const positiveKey = direction === "horizontal" ? "ArrowRight" : "ArrowDown";
          if (event.key === negativeKey || event.key === positiveKey) {
            event.preventDefault();
            const delta = event.key === negativeKey ? -step : step;
            setSizes((current) => resizePanels(current, panels, handleIndex, delta));
          }
        }}
        onPointerDown={(event) => {
          onPointerDown?.(event);
          if (event.defaultPrevented) return;
          startRef.current = {
            point: direction === "horizontal" ? event.clientX : event.clientY,
            sizes,
          };
          document.body.style.cursor = direction === "horizontal" ? "col-resize" : "row-resize";
          document.body.style.userSelect = "none";
          event.currentTarget.setPointerCapture(event.pointerId);
        }}
        data-direction={direction}
        {...props}
      >
        {withGrip ? (
          <span
            className={cn(
              "absolute rounded-full bg-text-dim/60 opacity-0 transition-opacity group-hover:opacity-100",
              direction === "horizontal"
                ? "top-1/2 left-1/2 h-10 w-0.5 -translate-x-1/2 -translate-y-1/2"
                : "top-1/2 left-1/2 h-0.5 w-10 -translate-x-1/2 -translate-y-1/2",
            )}
          />
        ) : null}
      </button>
    );
  },
);

ResizableHandle.displayName = "ResizableHandle";

const ResizablePanelGroup = React.forwardRef<HTMLDivElement, ResizablePanelGroupProps>(
  ({ children, className, direction = "horizontal", ...props }, ref) => {
    const containerRef = React.useRef<HTMLDivElement>(null);
    const panels = React.useMemo(
      () =>
        React.Children.toArray(children).flatMap((child) =>
          !React.isValidElement<ResizablePanelProps>(child) || child.type !== ResizablePanel
            ? []
            : [
                {
                  defaultSize: child.props.defaultSize,
                  maxSize: child.props.maxSize,
                  minSize: child.props.minSize,
                },
              ],
        ),
      [children],
    );
    const [sizes, setSizes] = React.useState(() => buildSizes(panels));

    React.useEffect(
      () =>
        setSizes((current) => (current.length === panels.length ? current : buildSizes(panels))),
      [panels],
    );

    let panelIndex = 0;
    let handleIndex = 0;
    const items = React.Children.map(children, (child) => {
      if (!React.isValidElement(child)) return child;
      if (child.type === ResizablePanel)
        return React.cloneElement(child as React.ReactElement<InternalPanelProps>, {
          panelIndex: panelIndex++,
        });
      if (child.type === ResizableHandle)
        return React.cloneElement(child as React.ReactElement<InternalHandleProps>, {
          handleIndex: handleIndex++,
        });
      return child;
    });

    return (
      <ResizableContext.Provider value={{ containerRef, direction, panels, setSizes, sizes }}>
        <div
          ref={(node) => {
            containerRef.current = node;
            assignRef(ref, node);
          }}
          className={cn(
            "flex h-full min-h-0 w-full min-w-0",
            direction === "horizontal" ? "flex-row" : "flex-col",
            className,
          )}
          data-direction={direction}
          {...props}
        >
          {items}
        </div>
      </ResizableContext.Provider>
    );
  },
);

ResizablePanelGroup.displayName = "ResizablePanelGroup";

export { ResizableHandle, ResizablePanel, ResizablePanelGroup };
