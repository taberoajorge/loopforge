import { useEffect, useState } from "react";
import { Outlet } from "react-router";
import { isPermissionGranted, requestPermission } from "@tauri-apps/plugin-notification";
import { NotificationPanel } from "../components/NotificationPanel";
import { TitleBar } from "../components/TitleBar";
import { AppSidebar } from "../components/layout/AppSidebar";
import { cn } from "../lib/utils";
import {
  SidebarInset,
  SidebarLayout,
  SidebarProvider,
  SidebarRail,
} from "../components/ui/sidebar";
import { useNotificationStore } from "../stores/notificationStore";
import { useProjectListSync } from "../hooks/useProjectListSync";
import { useNotificationIngestion } from "../hooks/useNotificationIngestion";

function useNativeNotificationPermission() {
  useEffect(() => {
    isPermissionGranted()
      .then((granted) => {
        if (!granted) return requestPermission();
      })
      .catch(() => {});
  }, []);
}

export function AppLayout() {
  const [isNotificationPanelOpen, setIsNotificationPanelOpen] = useState(false);
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false);
  const isMac =
    typeof navigator !== "undefined" &&
    /Mac|iPhone|iPad|iPod/.test(navigator.userAgent);
  const totalUnread = useNotificationStore(
    (state) => state.notifications.filter((notif) => !notif.read).length,
  );

  useProjectListSync();
  useNotificationIngestion();
  useNativeNotificationPermission();

  return (
    <div
      className={cn(
        "flex h-full flex-col overflow-hidden border border-border bg-void text-text font-mono",
        isMac ? "rounded-2xl" : "rounded-lg",
      )}
    >
      <TitleBar />
      <SidebarProvider
        collapsed={isSidebarCollapsed}
        onCollapsedChange={setIsSidebarCollapsed}
      >
        <SidebarLayout>
          <AppSidebar
            totalUnread={totalUnread}
            onToggleNotifications={() =>
              setIsNotificationPanelOpen((openState) => !openState)
            }
          />
          <SidebarRail aria-label="Toggle navigation rail" />
          <SidebarInset className="min-h-0">
            <div className="min-h-0 flex-1 overflow-y-auto">
              <Outlet />
            </div>
          </SidebarInset>
          <NotificationPanel
            isOpen={isNotificationPanelOpen}
            onOpenChange={setIsNotificationPanelOpen}
          />
        </SidebarLayout>
      </SidebarProvider>
    </div>
  );
}
