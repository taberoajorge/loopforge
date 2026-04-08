import type { StatusBadgeStatus } from "../components/StatusBadge";
import type { Project } from "../types/project";

export const ACTIVE_PROJECT_STATUSES: ReadonlyArray<Project["status"]> = [
  "active",
  "paused",
  "blocked",
];

export const FINISHED_PROJECT_STATUSES: ReadonlyArray<Project["status"]> = [
  "completed",
  "failed",
];

export const PROJECT_STATUS_META: Record<
  Project["status"],
  { badgeStatus: StatusBadgeStatus; cardLabel: string; sidebarLabel: string }
> = {
  active: { badgeStatus: "running", cardLabel: "Running", sidebarLabel: "Running" },
  paused: { badgeStatus: "paused", cardLabel: "Paused", sidebarLabel: "Paused" },
  blocked: { badgeStatus: "blocked", cardLabel: "Blocked", sidebarLabel: "Blocked" },
  completed: { badgeStatus: "completed", cardLabel: "Completed", sidebarLabel: "Completed" },
  failed: { badgeStatus: "failed", cardLabel: "Failed", sidebarLabel: "Failed" },
  draft: { badgeStatus: "draft", cardLabel: "Draft", sidebarLabel: "Draft" },
  archived: { badgeStatus: "archived", cardLabel: "Archived", sidebarLabel: "Archived" },
};
