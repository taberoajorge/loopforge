export type ProjectStatus =
  | "draft"
  | "active"
  | "paused"
  | "completed"
  | "failed"
  | "blocked"
  | "archived";

export interface Project {
  id: string;
  name: string;
  description: string;
  status: ProjectStatus;
  workingDirectory: string;
  createdAt: string;
  updatedAt: string;
  storiesCompleted?: number;
  totalStories?: number;
  currentAgent?: string | null;
  sessionStartedAt?: string | null;
  sessionEndedAt?: string | null;
  durationLabel?: string | null;
  uptimeLabel?: string | null;
  wizardStep?: string | null;
}
