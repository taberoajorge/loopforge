import { useEffect } from "react";
import { onNotificationAdded } from "../lib/tauri";
import { useNotificationStore } from "../stores/notificationStore";

export function useNotificationIngestion() {
  const fetchNotifications = useNotificationStore((state) => state.fetchNotifications);
  const ingestNotification = useNotificationStore((state) => state.ingestNotification);

  useEffect(() => {
    void fetchNotifications();
    const unlisten = onNotificationAdded((payload) => {
      ingestNotification(payload);
    });
    return () => {
      unlisten.then((off) => off());
    };
  }, [fetchNotifications, ingestNotification]);
}
