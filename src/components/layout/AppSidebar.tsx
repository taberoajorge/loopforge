import { useEffect, useState } from "react";
import { NavLink, useLocation, useNavigate } from "react-router";
import { Bell, ChevronDown, House, Plus } from "lucide-react";
import { ThemeToggle } from "../ThemeToggle";
import { Button } from "../ui/button";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "../ui/collapsible";
import { Sidebar, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupContent, SidebarGroupLabel, SidebarHeader, SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarMenuLabel, SidebarTrigger, useSidebar } from "../ui/sidebar";
import { useProjectStore, type Project } from "../../stores/projectStore";
import { useThemeStore } from "../../stores/themeStore";
import { useWizardStore } from "../../stores/wizardStore";
import { ACTIVE_PROJECT_STATUSES, FINISHED_PROJECT_STATUSES } from "../../lib/project-status";
import { ProjectMenuItem } from "./ProjectMenuItem";

type AppSidebarProps = { onToggleNotifications: () => void; totalUnread: number };
type ToggleSectionKey = "drafts" | "finished" | "archived";
const SIDEBAR_SECTION_KEY = "loopforge-sidebar-sections";

function readSavedSections(): Record<ToggleSectionKey, boolean> {
  if (typeof window === "undefined") {
    return { drafts: true, finished: true, archived: false };
  }
  const raw = window.localStorage.getItem(SIDEBAR_SECTION_KEY);
  if (!raw) {
    return { drafts: true, finished: true, archived: false };
  }
  try {
    const parsed = JSON.parse(raw) as Partial<Record<ToggleSectionKey, boolean>>;
    return {
      drafts: parsed.drafts ?? true,
      finished: parsed.finished ?? true,
      archived: parsed.archived ?? false,
    };
  } catch {
    return { drafts: true, finished: true, archived: false };
  }
}

export function AppSidebar({ onToggleNotifications, totalUnread }: AppSidebarProps) {
  const navigate = useNavigate();
  const location = useLocation();
  const projects = useProjectStore((state) => state.projects);
  const theme = useThemeStore((state) => state.theme);
  const setTheme = useThemeStore((state) => state.setTheme);
  const { collapsed } = useSidebar();
  const [sectionOpen, setSectionOpen] = useState<Record<ToggleSectionKey, boolean>>(readSavedSections);
  const activeProjects = projects.filter((project) => ACTIVE_PROJECT_STATUSES.includes(project.status));
  const draftProjects = projects.filter((project) => project.status === "draft");
  const finishedProjects = projects.filter((project) => FINISHED_PROJECT_STATUSES.includes(project.status));
  const archivedProjects = projects.filter((project) => project.status === "archived");
  const collapsedProjects = activeProjects;

  useEffect(() => {
    if (typeof window === "undefined") return;
    window.localStorage.setItem(SIDEBAR_SECTION_KEY, JSON.stringify(sectionOpen));
  }, [sectionOpen]);

  function renderProjectList(list: Project[], emptyLabel: string) {
    if (list.length === 0) {
      if (collapsed) {
        return null;
      }
      return <p className="px-2 py-1 text-xs text-sidebar-foreground/55">{emptyLabel}</p>;
    }
    return (
      <SidebarMenu>
        {list.map((project) => <ProjectMenuItem key={project.id} currentPath={location.pathname} project={project} />)}
      </SidebarMenu>
    );
  }

  function renderToggleGroup(section: ToggleSectionKey, label: string, list: Project[]) {
    if (list.length === 0) return null;
    const isOpen = sectionOpen[section];
    return (
      <Collapsible
        open={isOpen}
        onOpenChange={(open) => setSectionOpen((state) => ({ ...state, [section]: open }))}
        className="space-y-1"
      >
        <CollapsibleTrigger asChild>
          <button className="ui-type-label flex w-full items-center justify-between px-2 font-sans uppercase tracking-[0.18em] text-sidebar-foreground/55">
            <span>{label}</span>
            <ChevronDown className={`h-3 w-3 transition-transform ${isOpen ? "" : "-rotate-90"}`} />
          </button>
        </CollapsibleTrigger>
        <CollapsibleContent>
          <div className="ml-2 border-l border-sidebar-border/60 pl-2">
            {renderProjectList(list, `No ${label.toLowerCase()}`)}
          </div>
        </CollapsibleContent>
      </Collapsible>
    );
  }

  return (
    <Sidebar className="overflow-x-hidden">
      <SidebarHeader className={collapsed ? "gap-1 p-1" : "gap-2"}>
        <div className={collapsed ? "flex items-center justify-between" : "flex min-w-0 flex-1 items-center gap-2"}>
          <SidebarTrigger
            aria-label="Toggle navigation"
            className={collapsed ? "h-6 w-6" : "shrink-0"}
          />
          {collapsed ? null : (
            <NavLink to="/" className="flex min-w-0 flex-1 items-center gap-2 rounded-md px-2 py-1 hover:bg-sidebar-accent">
              <span className="text-xs font-bold tracking-[0.12em] text-sidebar-primary">LF</span>
              <span className="truncate text-xs font-sans text-sidebar-foreground/70">
                AI Loop Orchestrator
              </span>
            </NavLink>
          )}
          <Button
            variant="ghost"
            size="icon"
            className={collapsed ? "relative h-6 w-6 text-sidebar-foreground/70 hover:text-sidebar-foreground" : "relative shrink-0 text-sidebar-foreground/70 hover:text-sidebar-foreground"}
            onClick={onToggleNotifications}
            aria-label="Toggle notifications"
          >
            <Bell className={collapsed ? "h-3.5 w-3.5" : "h-4 w-4"} />
            {totalUnread > 0 ? (
              collapsed
                ? <span className="absolute right-0.5 top-0.5 h-1.5 w-1.5 rounded-full bg-blocked" />
                : <span className="absolute right-0 top-0 min-w-[0.9rem] rounded-full bg-blocked px-1 py-0.5 text-[10px] leading-none text-void">{totalUnread}</span>
            ) : null}
          </Button>
        </div>
      </SidebarHeader>
      <SidebarContent className="space-y-5 overflow-x-hidden">
        <SidebarGroup className="space-y-2">
          <SidebarGroupLabel className="ui-type-title tracking-[0.12em] text-sidebar-foreground/75">
            Workspace
          </SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              <SidebarMenuItem>
                <SidebarMenuButton active={location.pathname === "/"} onClick={() => navigate("/")}>
                  <House className="h-4 w-4 shrink-0" />
                  <SidebarMenuLabel>Home</SidebarMenuLabel>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
        <SidebarGroup className="space-y-3">
          <SidebarGroupLabel className="ui-type-title tracking-[0.12em] text-sidebar-foreground/75">
            Projects
          </SidebarGroupLabel>
          <SidebarGroupContent className="space-y-3">
            {projects.length === 0 ? (
              <p className="px-2 py-1 text-xs text-sidebar-foreground/55">No projects yet</p>
            ) : collapsed ? (
              renderProjectList(collapsedProjects, "No projects")
            ) : (
              <>
                <div className="space-y-1">
                  <p className="ui-type-label px-2 font-sans uppercase tracking-[0.18em] text-sidebar-foreground/55">
                    Active
                  </p>
                  <div className="ml-2 border-l border-sidebar-border/60 pl-2">
                    {renderProjectList(activeProjects, "No active loops")}
                  </div>
                </div>
                {renderToggleGroup("drafts", "Drafts", draftProjects)}
                {renderToggleGroup("finished", "Finished", finishedProjects)}
                {renderToggleGroup("archived", "Archived", archivedProjects)}
              </>
            )}
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter className="space-y-2">
        {collapsed ? null : (
          <ThemeToggle
            value={theme}
            onValueChange={setTheme}
            label="Theme"
            description="Switch light/dark"
            showModeBadge
            className="w-full rounded-md border border-sidebar-border bg-sidebar-accent/30 px-3 py-2"
          />
        )}
        <Button
          variant="primary"
          size={collapsed ? "icon" : "sm"}
          fullWidth={!collapsed}
          onClick={() => {
            useWizardStore.getState().reset();
            navigate("/new/describe");
          }}
        >
          <Plus className="h-4 w-4" />
          {collapsed ? null : "New Project"}
        </Button>
      </SidebarFooter>
    </Sidebar>
  );
}
