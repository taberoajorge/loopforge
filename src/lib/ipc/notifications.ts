import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type RingColor = "red" | "cyan" | "amber" | "green";

export interface AppNotification {
  id: string;
  projectId: string;
  notificationType: string;
  title: string;
  message: string;
  ringColor: RingColor;
  read: boolean;
  timestamp: number;
}

export interface NotificationListResponse {
  notifications: AppNotification[];
  unreadCount: number;
  ringColor: RingColor | null;
  projectSummaries: ProjectNotificationSummary[];
}

export interface NotificationAddedPayload {
  projectId: string;
  notification: AppNotification;
}

export interface ProjectNotificationSummary {
  projectId: string;
  unreadCount: number;
  ringColor: RingColor | null;
}

export async function addNotification(args: {
  projectId: string;
  notificationType: string;
  title: string;
  message: string;
}): Promise<AppNotification> {
  return invoke<AppNotification>("add_notification", { args });
}

export async function getNotifications(projectId?: string): Promise<NotificationListResponse> {
  return invoke<NotificationListResponse>("get_notifications", { projectId: projectId ?? null });
}

export async function markNotificationRead(notificationId: string): Promise<void> {
  return invoke("mark_notification_read", { notificationId });
}

export async function markAllNotificationsRead(projectId?: string): Promise<void> {
  return invoke("mark_all_notifications_read", { projectId: projectId ?? null });
}

export async function clearNotificationsBackend(projectId?: string): Promise<void> {
  return invoke("clear_notifications", { projectId: projectId ?? null });
}

export function onNotificationAdded(
  callback: (payload: NotificationAddedPayload) => void,
): Promise<UnlistenFn> {
  return listen<NotificationAddedPayload>("notification:added", (event) => callback(event.payload));
}
