import { create } from "zustand";
import { reportError } from "../lib/reportError";
import {
  addNotification as addNotificationBackend,
  type BackendNotification,
  clearNotificationsBackend,
  getNotifications,
  markAllNotificationsRead as markAllReadBackend,
  markNotificationRead as markReadBackend,
  type NotificationAddedPayload,
  type NotificationListResponse,
  type ProjectNotificationSummary,
  type RingColor,
} from "../lib/tauri";

export type AppNotification = BackendNotification;

export type NotificationType =
  | "story_blocked"
  | "loop_completed"
  | "rate_limited"
  | "review_comment"
  | "loop_error"
  | "story_completed";

export type { RingColor };

type ProjectSummaryMap = Record<string, ProjectNotificationSummary>;

interface NotificationState {
  notifications: AppNotification[];
  loading: boolean;
  unreadCount: number;
  ringColor: RingColor | null;
  projectSummaries: ProjectSummaryMap;
  ingestNotification: (payload: NotificationAddedPayload) => void;
  fetchNotifications: (projectId?: string) => Promise<void>;
  addNotification: (partial: {
    projectId: string;
    type: NotificationType;
    title: string;
    message: string;
  }) => Promise<void>;
  markAsRead: (notificationId: string) => Promise<void>;
  markAllAsRead: (projectId?: string) => Promise<void>;
  clearNotifications: (projectId?: string) => Promise<void>;
  totalUnreadCount: () => number;
  unreadCountForProject: (projectId: string) => number;
  ringColorForProject: (projectId: string) => RingColor | null;
  getNotificationsForProject: (projectId: string) => AppNotification[];
}

function toProjectSummaryMap(summaries: ProjectNotificationSummary[]): ProjectSummaryMap {
  const map: ProjectSummaryMap = {};
  for (const summary of summaries) {
    map[summary.projectId] = summary;
  }
  return map;
}

function applyNotificationResponse(
  response: NotificationListResponse,
): Pick<NotificationState, "notifications" | "unreadCount" | "ringColor" | "projectSummaries"> {
  return {
    notifications: response.notifications,
    unreadCount: response.unreadCount,
    ringColor: response.ringColor,
    projectSummaries: toProjectSummaryMap(response.projectSummaries),
  };
}

const RING_PRIORITY: Record<RingColor, number> = {
  red: 4,
  amber: 3,
  cyan: 2,
  green: 1,
};

function strongerRingColor(current: RingColor | null, incoming: RingColor): RingColor {
  if (!current) return incoming;
  return RING_PRIORITY[incoming] >= RING_PRIORITY[current] ? incoming : current;
}

export const useNotificationStore = create<NotificationState>()((set, get) => ({
  notifications: [],
  loading: false,
  unreadCount: 0,
  ringColor: null,
  projectSummaries: {},
  ingestNotification: (payload) =>
    set((state) => {
      const notification = payload.notification;
      const notifications = [notification, ...state.notifications];
      const currentSummary = state.projectSummaries[payload.projectId] ?? {
        projectId: payload.projectId,
        unreadCount: 0,
        ringColor: null,
      };
      const nextProjectSummary = {
        projectId: payload.projectId,
        unreadCount: currentSummary.unreadCount + (notification.read ? 0 : 1),
        ringColor: strongerRingColor(currentSummary.ringColor, notification.ringColor),
      };
      return {
        notifications,
        unreadCount: state.unreadCount + (notification.read ? 0 : 1),
        ringColor: strongerRingColor(state.ringColor, notification.ringColor),
        projectSummaries: {
          ...state.projectSummaries,
          [payload.projectId]: nextProjectSummary,
        },
      };
    }),

  fetchNotifications: async (projectId) => {
    set({ loading: true });
    try {
      const response = await getNotifications(projectId);
      set(applyNotificationResponse(response));
    } catch (caughtError: unknown) {
      reportError("notificationStore.fetchNotifications", caughtError);
    }
    set({ loading: false });
  },

  addNotification: async (partial) => {
    await addNotificationBackend({
      projectId: partial.projectId,
      notificationType: partial.type,
      title: partial.title,
      message: partial.message,
    }).catch((caughtError: unknown) => {
      reportError("notificationStore.addNotification", caughtError);
    });
  },

  markAsRead: async (notificationId) => {
    await markReadBackend(notificationId).catch((caughtError: unknown) => {
      reportError("notificationStore.markAsRead", caughtError);
    });
    await get().fetchNotifications();
  },

  markAllAsRead: async (projectId) => {
    await markAllReadBackend(projectId).catch((caughtError: unknown) => {
      reportError("notificationStore.markAllAsRead", caughtError);
    });
    await get().fetchNotifications();
  },

  clearNotifications: async (projectId) => {
    await clearNotificationsBackend(projectId).catch((caughtError: unknown) => {
      reportError("notificationStore.clearNotifications", caughtError);
    });
    await get().fetchNotifications();
  },

  totalUnreadCount: () => get().unreadCount,
  unreadCountForProject: (projectId) => get().projectSummaries[projectId]?.unreadCount ?? 0,
  ringColorForProject: (projectId) => get().projectSummaries[projectId]?.ringColor ?? null,
  getNotificationsForProject: (projectId) =>
    get().notifications.filter((notif) => notif.projectId === projectId),
}));
