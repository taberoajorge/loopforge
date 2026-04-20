import { cn } from "@/lib/utils";
import { useDisplayVocabularyStore } from "@/stores/displayVocabularyStore";
import type { AppNotification, NotificationType, RingColor } from "../../stores/notificationStore";
import { Badge, type BadgeProps } from "../ui/badge";
import { Button } from "../ui/button";
import { ScrollArea, ScrollContent, ScrollViewport } from "../ui/scroll-area";
import { SheetDescription, SheetHeader, SheetTitle } from "../ui/sheet";

type NotificationPanelLayoutProps = {
  notifications: AppNotification[];
  unreadCount: number;
  onMarkAllAsRead: () => void;
  onSelectNotification: (notification: AppNotification) => void;
};

function formatTimeAgo(timestamp: number): string {
  const secondsElapsed = Math.floor((Date.now() - timestamp) / 1000);
  if (secondsElapsed < 60) return "just now";
  const minutesElapsed = Math.floor(secondsElapsed / 60);
  if (minutesElapsed < 60) return `${minutesElapsed}m ago`;
  const hoursElapsed = Math.floor(minutesElapsed / 60);
  if (hoursElapsed < 24) return `${hoursElapsed}h ago`;
  const daysElapsed = Math.floor(hoursElapsed / 24);
  return `${daysElapsed}d ago`;
}

function NotificationRow({
  notification,
  ringVariantMap,
  typeLabelMap,
  onSelectNotification,
}: {
  notification: AppNotification;
  ringVariantMap: Record<string, string>;
  typeLabelMap: Record<string, string>;
  onSelectNotification: (notification: AppNotification) => void;
}) {
  return (
    <button
      type="button"
      className={cn(
        "w-full border-border/50 border-b px-4 py-3 text-left transition-colors hover:bg-elevated/60",
        notification.read ? "opacity-70" : "opacity-100",
      )}
      onClick={() => onSelectNotification(notification)}
    >
      <div className="flex items-center justify-between gap-2">
        <p className="truncate font-sans text-sm text-text">{notification.title}</p>
        <Badge
          variant={
            (ringVariantMap[notification.ringColor as RingColor] ?? "info") as NonNullable<
              BadgeProps["variant"]
            >
          }
        >
          {typeLabelMap[notification.notificationType as NotificationType] ??
            notification.notificationType}
        </Badge>
      </div>
      <p className="mt-1 line-clamp-2 font-sans text-text-muted text-xs">{notification.message}</p>
      <div className="mt-2 flex items-center justify-between">
        <p className="font-mono text-text-dim text-xs">{formatTimeAgo(notification.timestamp)}</p>
        {!notification.read ? (
          <Badge variant="danger" emphasis="solid">
            New
          </Badge>
        ) : null}
      </div>
    </button>
  );
}

export function NotificationPanelLayout({
  notifications,
  unreadCount,
  onMarkAllAsRead,
  onSelectNotification,
}: NotificationPanelLayoutProps) {
  const vocabulary = useDisplayVocabularyStore((state) => state.vocabulary);
  const ringVariantMap = vocabulary?.notificationRingVariants ?? {};
  const typeLabelMap = vocabulary?.notificationTypeLabels ?? {};

  return (
    <div className="flex h-full min-h-0 flex-col">
      <SheetHeader className="gap-3 border-border border-b px-4 py-3 pr-12">
        <div className="flex items-center justify-between gap-3">
          <SheetTitle className="text-base">Notifications</SheetTitle>
          {unreadCount > 0 ? (
            <Badge variant="danger" emphasis="solid">
              {unreadCount}
            </Badge>
          ) : null}
        </div>
        <div className="flex items-center justify-between gap-3">
          <SheetDescription>Recent loop and project activity</SheetDescription>
          {unreadCount > 0 ? (
            <Button variant="ghost" size="sm" onClick={onMarkAllAsRead}>
              Mark all read
            </Button>
          ) : null}
        </div>
      </SheetHeader>
      <ScrollArea className="flex-1">
        <ScrollViewport orientation="vertical" className="h-full">
          <ScrollContent>
            {notifications.length === 0 ? (
              <div className="flex h-full min-h-40 items-center justify-center px-4 py-12">
                <p className="font-sans text-sm text-text-dim">No notifications</p>
              </div>
            ) : (
              notifications.map((notification) => (
                <NotificationRow
                  key={notification.id}
                  notification={notification}
                  ringVariantMap={ringVariantMap}
                  typeLabelMap={typeLabelMap}
                  onSelectNotification={onSelectNotification}
                />
              ))
            )}
          </ScrollContent>
        </ScrollViewport>
      </ScrollArea>
    </div>
  );
}
