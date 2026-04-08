import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";

type SidebarContextValue = { collapsed: boolean; setCollapsed: (value: boolean) => void; toggle: () => void };
const SidebarContext = React.createContext<SidebarContextValue | null>(null);

export function useSidebar() {
  const context = React.useContext(SidebarContext);
  if (!context) throw new Error("Sidebar components must be used within SidebarProvider");
  return context;
}

export type SidebarProviderProps = React.PropsWithChildren<{
  collapsed?: boolean;
  defaultCollapsed?: boolean;
  onCollapsedChange?: (collapsed: boolean) => void;
}>;

function SidebarProvider({ children, collapsed, defaultCollapsed = false, onCollapsedChange }: SidebarProviderProps) {
  const [uncontrolledCollapsed, setUncontrolledCollapsed] = React.useState(defaultCollapsed);
  const currentCollapsed = collapsed ?? uncontrolledCollapsed;
  const setCollapsed = React.useCallback((nextCollapsed: boolean) => {
    if (collapsed === undefined) setUncontrolledCollapsed(nextCollapsed);
    onCollapsedChange?.(nextCollapsed);
  }, [collapsed, onCollapsedChange]);

  return <SidebarContext.Provider value={{ collapsed: currentCollapsed, setCollapsed, toggle: () => setCollapsed(!currentCollapsed) }}>{children}</SidebarContext.Provider>;
}

const sidebarVariants = cva("flex h-full shrink-0 flex-col border-sidebar-border bg-sidebar text-sidebar-foreground transition-[width] duration-200", {
  variants: {
    collapsed: { true: "w-16", false: "w-72" },
    side: { left: "border-r", right: "order-last border-l" },
  },
  defaultVariants: { collapsed: false, side: "left" },
});

const sidebarMenuButtonVariants = cva("flex w-full items-center gap-3 rounded-md border px-3 py-2 text-left text-sm font-sans transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sidebar-ring focus-visible:ring-offset-2 focus-visible:ring-offset-sidebar", {
  variants: {
    active: {
      true: "border-sidebar-primary/20 bg-sidebar-primary/10 text-sidebar-primary",
      false: "border-transparent text-sidebar-foreground/80 hover:bg-sidebar-accent hover:text-sidebar-foreground",
    },
    collapsed: { true: "justify-center px-2", false: "" },
  },
  defaultVariants: { active: false, collapsed: false },
});

export type SidebarLayoutProps = React.HTMLAttributes<HTMLDivElement>;
export type SidebarProps = React.HTMLAttributes<HTMLDivElement> & VariantProps<typeof sidebarVariants>;
export type SidebarMenuButtonProps = React.ButtonHTMLAttributes<HTMLButtonElement> & VariantProps<typeof sidebarMenuButtonVariants>;

const SidebarLayout = React.forwardRef<HTMLDivElement, SidebarLayoutProps>(({ className, ...props }, ref) => {
  return <div ref={ref} className={cn("flex min-h-0 min-w-0 flex-1 overflow-hidden bg-background", className)} {...props} />;
});

SidebarLayout.displayName = "SidebarLayout";

const Sidebar = React.forwardRef<HTMLDivElement, SidebarProps>(({ className, collapsed, side, ...props }, ref) => {
  const context = useSidebar();
  const currentCollapsed = collapsed ?? context.collapsed;
  return <aside ref={ref} className={cn(sidebarVariants({ collapsed: currentCollapsed, side }), className)} data-collapsed={currentCollapsed} data-state={currentCollapsed ? "collapsed" : "expanded"} {...props} />;
});

Sidebar.displayName = "Sidebar";

const SidebarInset = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(({ className, ...props }, ref) => <main ref={ref} className={cn("flex min-w-0 flex-1 flex-col bg-background", className)} {...props} />);
const SidebarHeader = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(({ className, ...props }, ref) => <div ref={ref} className={cn("flex items-center gap-3 border-b border-sidebar-border p-3", className)} {...props} />);
const SidebarContent = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(({ className, ...props }, ref) => <div ref={ref} className={cn("flex min-h-0 flex-1 flex-col overflow-y-auto p-2", className)} {...props} />);
const SidebarFooter = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(({ className, ...props }, ref) => <div ref={ref} className={cn("border-t border-sidebar-border p-3", className)} {...props} />);
const SidebarGroup = React.forwardRef<HTMLElement, React.HTMLAttributes<HTMLElement>>(({ className, ...props }, ref) => <section ref={ref} className={cn("space-y-2", className)} {...props} />);
const SidebarGroupContent = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(({ className, ...props }, ref) => <div ref={ref} className={cn("space-y-1", className)} {...props} />);
const SidebarMenu = React.forwardRef<HTMLUListElement, React.HTMLAttributes<HTMLUListElement>>(({ className, ...props }, ref) => <ul ref={ref} className={cn("space-y-1", className)} {...props} />);
const SidebarMenuItem = React.forwardRef<HTMLLIElement, React.HTMLAttributes<HTMLLIElement>>(({ className, ...props }, ref) => <li ref={ref} className={cn("list-none", className)} {...props} />);

SidebarInset.displayName = "SidebarInset";
SidebarHeader.displayName = "SidebarHeader";
SidebarContent.displayName = "SidebarContent";
SidebarFooter.displayName = "SidebarFooter";
SidebarGroup.displayName = "SidebarGroup";
SidebarGroupContent.displayName = "SidebarGroupContent";
SidebarMenu.displayName = "SidebarMenu";
SidebarMenuItem.displayName = "SidebarMenuItem";

const SidebarGroupLabel = React.forwardRef<HTMLParagraphElement, React.HTMLAttributes<HTMLParagraphElement>>(({ className, ...props }, ref) => {
  const { collapsed } = useSidebar();
  return <p ref={ref} className={cn("px-2 text-[10px] font-sans uppercase tracking-[0.18em] text-sidebar-foreground/55", collapsed && "hidden", className)} data-state={collapsed ? "collapsed" : "expanded"} {...props} />;
});

SidebarGroupLabel.displayName = "SidebarGroupLabel";

const SidebarMenuButton = React.forwardRef<HTMLButtonElement, SidebarMenuButtonProps>(({ active, className, collapsed, type, ...props }, ref) => {
  const context = useSidebar();
  const currentCollapsed = collapsed ?? context.collapsed;
  return <button ref={ref} className={cn(sidebarMenuButtonVariants({ active, collapsed: currentCollapsed }), className)} data-active={active ?? false} data-state={active ? "active" : "inactive"} type={type ?? "button"} {...props} />;
});

SidebarMenuButton.displayName = "SidebarMenuButton";

const SidebarMenuLabel = React.forwardRef<HTMLSpanElement, React.HTMLAttributes<HTMLSpanElement>>(({ className, ...props }, ref) => {
  const { collapsed } = useSidebar();
  return <span ref={ref} className={cn("truncate", collapsed && "hidden", className)} data-state={collapsed ? "collapsed" : "expanded"} {...props} />;
});

SidebarMenuLabel.displayName = "SidebarMenuLabel";

const SidebarMenuBadge = React.forwardRef<HTMLSpanElement, React.HTMLAttributes<HTMLSpanElement>>(({ className, ...props }, ref) => {
  const { collapsed } = useSidebar();
  return <span ref={ref} className={cn("ml-auto rounded-full bg-sidebar-accent px-1.5 py-0.5 text-[10px] font-mono text-sidebar-foreground", collapsed && "hidden", className)} data-state={collapsed ? "collapsed" : "expanded"} {...props} />;
});

SidebarMenuBadge.displayName = "SidebarMenuBadge";

const SidebarTrigger = React.forwardRef<HTMLButtonElement, React.ButtonHTMLAttributes<HTMLButtonElement>>(({ className, onClick, type, ...props }, ref) => {
  const { collapsed, toggle } = useSidebar();
  return (
    <button
      ref={ref}
      className={cn("inline-flex h-9 w-9 items-center justify-center rounded-md border border-transparent text-sidebar-foreground/70 transition-colors hover:bg-sidebar-accent hover:text-sidebar-foreground", className)}
      onClick={(event) => {
        onClick?.(event);
        if (!event.defaultPrevented) toggle();
      }}
      type={type ?? "button"}
      {...props}
    >
      <span aria-hidden className="text-sm leading-none">{collapsed ? "›" : "‹"}</span>
    </button>
  );
});

SidebarTrigger.displayName = "SidebarTrigger";

const SidebarRail = React.forwardRef<HTMLButtonElement, React.ButtonHTMLAttributes<HTMLButtonElement>>(({ className, onClick, type, ...props }, ref) => {
  const { collapsed, toggle } = useSidebar();
  return (
    <button
      ref={ref}
      className={cn("flex h-full w-4 shrink-0 items-center justify-center border-r border-sidebar-border bg-sidebar-accent/40 text-sidebar-foreground/50 transition-colors hover:bg-sidebar-accent hover:text-sidebar-foreground", className)}
      data-state={collapsed ? "collapsed" : "expanded"}
      onClick={(event) => {
        onClick?.(event);
        if (!event.defaultPrevented) toggle();
      }}
      type={type ?? "button"}
      {...props}
    >
      <span aria-hidden className="text-[10px] leading-none">{collapsed ? "›" : "‹"}</span>
    </button>
  );
});

SidebarRail.displayName = "SidebarRail";

export {
  SidebarLayout,
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarInset,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuLabel,
  SidebarProvider,
  SidebarRail,
  SidebarTrigger,
};
