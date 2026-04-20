import { invoke } from "@tauri-apps/api/core";

export interface ProjectStatusMeta {
  badgeStatus: string;
  cardLabel: string;
  sidebarLabel: string;
}

export interface PlanKindMeta {
  label: string;
  variant: string;
}

export interface DisplayVocabulary {
  projectStatusMeta: Record<string, ProjectStatusMeta>;
  statusLabels: Record<string, string>;
  statusVariants: Record<string, string>;
  notificationTypeLabels: Record<string, string>;
  notificationRingVariants: Record<string, string>;
  storyStatusVariants: Record<string, string>;
  activityResultVariants: Record<string, string>;
  monitorStatusBadges: Record<string, string>;
  planKindMeta: Record<string, PlanKindMeta>;
  atomizeActivityKindMeta: Record<string, PlanKindMeta>;
  storyPriorityVariants: Record<string, string>;
  wizardStepLabels: Record<string, string>;
  stageStatusBadges: Record<string, string>;
  stageStatusLabels: Record<string, string>;
  stepIndicatorVariants: Record<string, string>;
  stepIndicatorEmphasis: Record<string, string>;
  agentNames: string[];
  inactiveStatuses: string[];
  stallThresholdSecs: number;
  maxVisibleActivityEvents: number;
  maxOutputLines: number;
}

export async function getDisplayVocabulary(): Promise<DisplayVocabulary> {
  return invoke<DisplayVocabulary>("get_display_vocabulary");
}
