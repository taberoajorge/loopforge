import { useNavigate } from "react-router";
import { Sheet, SheetContent } from "./ui/sheet";
import { NotificationPanelLayout } from "./notification-panel/NotificationPanelLayout";
import { useNotificationStore, type AppNotification } from "../stores/notificationStore";

interface NotificationPanelProps {
  isOpen: boolean;
  onOpenChange: (isOpen: boolean) => void;
}

export function NotificationPanel({ isOpen, onOpenChange }: NotificationPanelProps) {
  const navigate = useNavigate();
  const notifications = useNotificationStore((state) => state.notifications);
  const markAllAsRead = useNotificationStore((state) => state.markAllAsRead);
  const markAsRead = useNotificationStore((state) => state.markAsRead);
  const unreadCount = notifications.filter((notif) => !notif.read).length;

  function handleSelect(notification: AppNotification) {
    markAsRead(notification.id);
    onOpenChange(false);
    navigate(`/monitor/${notification.projectId}`);
  }

  return (
    <Sheet open={isOpen} onOpenChange={onOpenChange}>
      <SheetContent side="right" className="w-full p-0 sm:max-w-md">
        <NotificationPanelLayout
          notifications={notifications}
          unreadCount={unreadCount}
          onMarkAllAsRead={() => markAllAsRead()}
          onSelectNotification={handleSelect}
        />
      </SheetContent>
    </Sheet>
  );
}
