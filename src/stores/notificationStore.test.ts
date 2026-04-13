import { beforeEach, describe, expect, it, vi } from "vitest";
import { useNotificationStore } from "./notificationStore";

describe("notificationStore", () => {
  beforeEach(() => {
    useNotificationStore.setState({ notifications: [] });
    vi.spyOn(Date, "now").mockReturnValue(1_700_000_000_000);
    vi.spyOn(Math, "random").mockReturnValue(0.123456);
  });

  it("adds unread notifications with derived ring colors and counts", () => {
    const store = useNotificationStore.getState();

    store.addNotification({
      projectId: "project-001",
      type: "review_comment",
      title: "Review comment",
      message: "A reviewer asked for changes",
    });
    store.addNotification({
      projectId: "project-001",
      type: "loop_error",
      title: "Verification failed",
      message: "Tests failed",
    });

    expect(useNotificationStore.getState().notifications).toHaveLength(2);
    expect(useNotificationStore.getState().totalUnreadCount()).toBe(2);
    expect(useNotificationStore.getState().unreadCountForProject("project-001")).toBe(2);
    expect(useNotificationStore.getState().ringColorForProject("project-001")).toBe("red");
  });

  it("marks notifications as read and clears per-project history", () => {
    const store = useNotificationStore.getState();

    store.addNotification({
      projectId: "project-001",
      type: "story_completed",
      title: "Done",
      message: "Story passed",
    });
    store.addNotification({
      projectId: "project-002",
      type: "rate_limited",
      title: "Rate limited",
      message: "Agent switched",
    });

    const [firstId, secondId] = useNotificationStore
      .getState()
      .notifications.map((item) => item.id);

    store.markAsRead(firstId);
    expect(useNotificationStore.getState().unreadCountForProject("project-002")).toBe(0);

    store.markAllAsRead("project-001");
    expect(useNotificationStore.getState().totalUnreadCount()).toBe(0);

    store.clearNotifications("project-001");
    expect(useNotificationStore.getState().getNotificationsForProject("project-001")).toEqual([]);
    expect(useNotificationStore.getState().getNotificationsForProject("project-002")).toHaveLength(
      1,
    );
    expect(secondId).toMatch(/^notif_/);
  });
});
