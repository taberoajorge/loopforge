import * as React from "react";

import { cn } from "@/lib/utils";

type Orientation = "horizontal" | "vertical";
type TabsContextValue = {
  contentId: (value: string) => string;
  orientation: Orientation;
  setValue: (value: string) => void;
  triggerId: (value: string) => string;
  value: string;
};

const TabsContext = React.createContext<TabsContextValue | null>(null);

function useTabsContext(component: string) {
  const context = React.useContext(TabsContext);
  if (!context) throw new Error(`${component} must be used within Tabs`);
  return context;
}

export type TabsProps = React.HTMLAttributes<HTMLDivElement> & {
  defaultValue?: string;
  onValueChange?: (value: string) => void;
  orientation?: Orientation;
  value?: string;
};

const Tabs = React.forwardRef<HTMLDivElement, TabsProps>(
  ({ children, className, defaultValue = "", onValueChange, orientation = "horizontal", value, ...props }, ref) => {
    const id = React.useId();
    const [uncontrolledValue, setUncontrolledValue] = React.useState(defaultValue);
    const currentValue = value ?? uncontrolledValue;

    const setValue = React.useCallback((nextValue: string) => {
      if (value === undefined) setUncontrolledValue(nextValue);
      onValueChange?.(nextValue);
    }, [onValueChange, value]);

    return (
      <TabsContext.Provider
        value={{
          contentId: (itemValue) => `${id}-${itemValue}-content`,
          orientation,
          setValue,
          triggerId: (itemValue) => `${id}-${itemValue}-trigger`,
          value: currentValue,
        }}
      >
        <div
          ref={ref}
          className={cn("flex min-h-0 min-w-0 gap-4", orientation === "horizontal" ? "flex-col" : "flex-row", className)}
          data-orientation={orientation}
          {...props}
        >
          {children}
        </div>
      </TabsContext.Provider>
    );
  },
);

Tabs.displayName = "Tabs";

export type TabsListProps = React.HTMLAttributes<HTMLDivElement>;

const TabsList = React.forwardRef<HTMLDivElement, TabsListProps>(({ className, ...props }, ref) => {
  const { orientation } = useTabsContext("TabsList");
  return (
    <div
      ref={ref}
      role="tablist"
      aria-orientation={orientation}
      className={cn("inline-flex w-fit items-center gap-1 rounded-lg border border-border bg-surface p-1", orientation === "vertical" && "flex-col items-stretch", className)}
      {...props}
    />
  );
});

TabsList.displayName = "TabsList";

export type TabsTriggerProps = Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "value"> & {
  value: string;
};

const TabsTrigger = React.forwardRef<HTMLButtonElement, TabsTriggerProps>(
  ({ className, onClick, onKeyDown, value, ...props }, ref) => {
    const context = useTabsContext("TabsTrigger");
    const active = context.value === value;

    function focusTrigger(event: React.KeyboardEvent<HTMLButtonElement>, nextIndex: number) {
      const list = event.currentTarget.closest("[role='tablist']");
      if (!list) return;
      const triggers = Array.from(list.querySelectorAll<HTMLButtonElement>("[role='tab']:not(:disabled)"));
      if (!triggers.length) return;
      const nextTrigger = triggers[nextIndex];
      nextTrigger?.focus();
      const nextValue = nextTrigger?.dataset.value;
      if (nextValue) context.setValue(nextValue);
    }

    return (
      <button
        ref={ref}
        aria-controls={context.contentId(value)}
        aria-selected={active}
        className={cn(
          "inline-flex items-center justify-center rounded-md border px-3 py-2 text-xs font-sans font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-50",
          active ? "border-primary/30 bg-primary/10 text-primary shadow-glow-primary" : "border-transparent text-text-muted hover:bg-elevated hover:text-text",
          className,
        )}
        data-state={active ? "active" : "inactive"}
        data-value={value}
        id={context.triggerId(value)}
        onClick={(event) => {
          onClick?.(event);
          if (!event.defaultPrevented) context.setValue(value);
        }}
        onKeyDown={(event) => {
          onKeyDown?.(event);
          if (event.defaultPrevented) return;
          const isHorizontal = context.orientation === "horizontal";
          const list = event.currentTarget.closest("[role='tablist']");
          const triggers = list ? Array.from(list.querySelectorAll<HTMLButtonElement>("[role='tab']:not(:disabled)")) : [];
          const index = triggers.indexOf(event.currentTarget);
          if (!triggers.length || index === -1) return;
          if ((isHorizontal && event.key === "ArrowRight") || (!isHorizontal && event.key === "ArrowDown")) {
            event.preventDefault();
            focusTrigger(event, (index + 1 + triggers.length) % triggers.length);
          } else if ((isHorizontal && event.key === "ArrowLeft") || (!isHorizontal && event.key === "ArrowUp")) {
            event.preventDefault();
            focusTrigger(event, (index - 1 + triggers.length) % triggers.length);
          } else if (event.key === "Home" || event.key === "End") {
            event.preventDefault();
            focusTrigger(event, event.key === "Home" ? 0 : triggers.length - 1);
          }
        }}
        role="tab"
        tabIndex={active ? 0 : -1}
        type="button"
        {...props}
      />
    );
  },
);

TabsTrigger.displayName = "TabsTrigger";

export type TabsContentProps = Omit<React.HTMLAttributes<HTMLDivElement>, "value"> & {
  forceMount?: boolean;
  value: string;
};

const TabsContent = React.forwardRef<HTMLDivElement, TabsContentProps>(({ className, forceMount = false, value, ...props }, ref) => {
  const context = useTabsContext("TabsContent");
  const active = context.value === value;
  if (!active && !forceMount) return null;

  return (
    <div
      ref={ref}
      aria-labelledby={context.triggerId(value)}
      className={cn("min-h-0 min-w-0 flex-1 outline-none", !active && "hidden", className)}
      data-state={active ? "active" : "inactive"}
      hidden={!active}
      id={context.contentId(value)}
      role="tabpanel"
      tabIndex={0}
      {...props}
    />
  );
});

TabsContent.displayName = "TabsContent";

export { Tabs, TabsContent, TabsList, TabsTrigger };
