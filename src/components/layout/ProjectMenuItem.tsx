import { useNavigate } from "react-router";
import { cn } from "../../lib/utils";
import { useDisplayVocabularyStore } from "../../stores/displayVocabularyStore";
import { type RingColor, useNotificationStore } from "../../stores/notificationStore";
import type { Project } from "../../stores/projectStore";
import { StatusBadge, type StatusBadgeStatus } from "../StatusBadge";
import { SidebarMenuButton, SidebarMenuItem, SidebarMenuLabel, useSidebar } from "../ui/sidebar";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "../ui/tooltip";

const RING_CLASSES: Record<RingColor, string> = {
  red: "border-l-2 border-l-blocked",
  amber: "border-l-2 border-l-paused",
  cyan: "border-l-2 border-l-primary",
  green: "border-l-2 border-l-success",
};

const STATUS_DOT: Record<string, string> = {
  running: "bg-running",
  paused: "bg-paused",
  blocked: "bg-blocked",
  completed: "bg-success",
  failed: "bg-blocked",
  draft: "bg-text-dim",
  archived: "bg-muted-foreground",
};

function projectHref(project: Project): string {
  if (project.status === "draft") {
    const step = project.wizardStep ?? "describe";
    return `/new/${step}/${project.id}`;
  }
  return `/monitor/${project.id}`;
}

type ProjectMenuItemProps = { currentPath: string; project: Project };

export function ProjectMenuItem({ currentPath, project }: ProjectMenuItemProps) {
  const navigate = useNavigate();
  const { collapsed } = useSidebar();
  const href = projectHref(project);
  const vocabulary = useDisplayVocabularyStore((state) => state.vocabulary);
  const statusMeta = vocabulary?.projectStatusMeta[project.status] ?? {
    badgeStatus: project.status === "active" ? "running" : project.status,
    cardLabel: `${project.status.slice(0, 1).toUpperCase()}${project.status.slice(1)}`,
    sidebarLabel: `${project.status.slice(0, 1).toUpperCase()}${project.status.slice(1)}`,
  };
  const draftSubLabel =
    project.status === "draft" && project.wizardStep
      ? (vocabulary?.wizardStepLabels[project.wizardStep] ?? project.wizardStep)
      : null;
  const sidebarLabel = draftSubLabel ? `Draft · ${draftSubLabel}` : statusMeta.sidebarLabel;
  const ringColor = useNotificationStore((state) => state.ringColorForProject(project.id));
  const unreadCount = useNotificationStore((state) => state.unreadCountForProject(project.id));
  const isActive = currentPath === href || currentPath.startsWith(`${href}/`);
  const tooltipText = `${project.name} · ${sidebarLabel}`;

  if (collapsed) {
    return (
      <SidebarMenuItem>
        <TooltipProvider delayDuration={200}>
          <Tooltip>
            <TooltipTrigger asChild>
              <SidebarMenuButton
                active={isActive}
                className={cn(
                  "!px-0 relative flex items-center justify-center",
                  ringColor ? RING_CLASSES[ringColor] : "",
                )}
                onClick={() => navigate(href)}
              >
                <span className="font-mono font-semibold text-xs uppercase leading-none">
                  {project.name.slice(0, 2) || "??"}
                </span>
                <span
                  className={cn(
                    "absolute right-0.5 bottom-0.5 h-2 w-2 rounded-full ring-1 ring-sidebar",
                    STATUS_DOT[statusMeta.badgeStatus] ?? "bg-text-dim",
                  )}
                />
                {unreadCount > 0 ? (
                  <span className="absolute top-0 right-0 h-1.5 w-1.5 rounded-full bg-blocked" />
                ) : null}
              </SidebarMenuButton>
            </TooltipTrigger>
            <TooltipContent side="right" className="text-xs">
              {tooltipText}
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      </SidebarMenuItem>
    );
  }

  return (
    <SidebarMenuItem>
      <TooltipProvider delayDuration={400}>
        <Tooltip>
          <TooltipTrigger asChild>
            <SidebarMenuButton
              active={isActive}
              className={cn(ringColor ? RING_CLASSES[ringColor] : "")}
              onClick={() => navigate(href)}
            >
              <SidebarMenuLabel className="flex min-w-0 flex-1 flex-col items-start overflow-hidden">
                <span className="block w-full truncate text-sm">{project.name}</span>
                <span className="font-mono text-[10px] text-sidebar-foreground/55">
                  {project.id.slice(0, 8)}
                </span>
              </SidebarMenuLabel>
              {unreadCount > 0 ? (
                <span className="h-1.5 w-1.5 shrink-0 rounded-full bg-blocked" />
              ) : null}
              <StatusBadge
                status={statusMeta.badgeStatus as StatusBadgeStatus}
                label={sidebarLabel}
                className="ml-1 min-w-[4.5rem] justify-center text-center text-[10px]"
              />
            </SidebarMenuButton>
          </TooltipTrigger>
          <TooltipContent side="right" className="text-xs">
            {tooltipText}
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
      {project.totalStories != null && project.totalStories > 0 ? (
        <p className="mx-2 mt-0.5 font-mono text-[10px] text-sidebar-foreground/50">
          {project.storiesCompleted ?? 0}/{project.totalStories} stories
        </p>
      ) : null}
    </SidebarMenuItem>
  );
}
