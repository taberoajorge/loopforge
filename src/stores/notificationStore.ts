import { create } from "zustand";

export type NotificationType =
  | "story_blocked"
  | "loop_completed"
  | "rate_limited"
  | "review_comment"
  | "loop_error"
  | "story_completed";

export type RingColor = "red" | "cyan" | "amber" | "green";

export interface AppNotification {
  id: string;
  projectId: string;
  type: NotificationType;
  title: string;
  message: string;
  timestamp: number;
  read: boolean;
  ringColor: RingColor;
}

interface NotificationState {
  notifications: AppNotification[];
  addNotification: (notification: Omit<AppNotification, "id" | "timestamp" | "read" | "ringColor"> & { ringColor?: RingColor }) => void;
  markAsRead: (notificationId: string) => void;
  markAllAsRead: (projectId?: string) => void;
  clearNotifications: (projectId?: string) => void;
  totalUnreadCount: () => number;
  unreadCountForProject: (projectId: string) => number;
  ringColorForProject: (projectId: string) => RingColor | null;
  getNotificationsForProject: (projectId: string) => AppNotification[];
}

function generateId(): string {
  return `notif_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
}

function typeToRingColor(type: NotificationType): RingColor {
  switch (type) {
    case "story_blocked":
    case "loop_error":
      return "red";
    case "loop_completed":
    case "story_completed":
      return "cyan";
    case "rate_limited":
    case "review_comment":
      return "amber";
    default:
      return "cyan";
  }
}

export const useNotificationStore = create<NotificationState>()((set, get) => ({
  notifications: [],

  addNotification: (partial) =>
    set((state) => ({
      notifications: [
        {
          ...partial,
          id: generateId(),
          timestamp: Date.now(),
          read: false,
          ringColor: partial.ringColor ?? typeToRingColor(partial.type),
        },
        ...state.notifications,
      ].slice(0, 200),
    })),

  markAsRead: (notificationId) =>
    set((state) => ({
      notifications: state.notifications.map((notif) =>
        notif.id === notificationId ? { ...notif, read: true } : notif,
      ),
    })),

  markAllAsRead: (projectId) =>
    set((state) => ({
      notifications: state.notifications.map((notif) =>
        !projectId || notif.projectId === projectId
          ? { ...notif, read: true }
          : notif,
      ),
    })),

  clearNotifications: (projectId) =>
    set((state) => ({
      notifications: projectId
        ? state.notifications.filter((notif) => notif.projectId !== projectId)
        : [],
    })),

  totalUnreadCount: () =>
    get().notifications.filter((notif) => !notif.read).length,

  unreadCountForProject: (projectId) =>
    get().notifications.filter(
      (notif) => notif.projectId === projectId && !notif.read,
    ).length,

  ringColorForProject: (projectId) => {
    const unread = get().notifications.filter(
      (notif) => notif.projectId === projectId && !notif.read,
    );
    if (unread.length === 0) return null;
    const priority: RingColor[] = ["red", "amber", "cyan", "green"];
    for (const color of priority) {
      if (unread.some((notif) => notif.ringColor === color)) return color;
    }
    return unread[0].ringColor;
  },

  getNotificationsForProject: (projectId) =>
    get().notifications.filter((notif) => notif.projectId === projectId),
}));
