import { isPermissionGranted, requestPermission } from "@tauri-apps/plugin-notification";
import { useEffect, useState } from "react";
import { Outlet } from "react-router";
import { AppSidebar } from "../components/layout/AppSidebar";
import { NotificationPanel } from "../components/NotificationPanel";
import { TitleBar } from "../components/TitleBar";
import {
  SidebarInset,
  SidebarLayout,
  SidebarProvider,
  SidebarRail,
} from "../components/ui/sidebar";
import { useNotificationIngestion } from "../hooks/useNotificationIngestion";
import { useProjectListSync } from "../hooks/useProjectListSync";
import { isApplePlatform } from "../lib/platform";
import { reportError } from "../lib/reportError";
import { cn } from "../lib/utils";
import { useAskStore } from "../stores/askStore";
import { useDisplayVocabularyStore } from "../stores/displayVocabularyStore";
import { useNotificationStore } from "../stores/notificationStore";
import { useWizardDefaultsStore } from "../stores/wizardDefaultsStore";
import { useWizardStore } from "../stores/wizardStore";

function useNativeNotificationPermission() {
  useEffect(() => {
    isPermissionGranted()
      .then((granted) => {
        if (!granted) return requestPermission();
      })
      .catch((caughtError: unknown) => {
        reportError("AppLayout.nativeNotificationPermission", caughtError);
      });
  }, []);
}

export function AppLayout() {
  const [isNotificationPanelOpen, setIsNotificationPanelOpen] = useState(false);
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false);
  const isMac = isApplePlatform();
  const totalUnread = useNotificationStore((state) => state.totalUnreadCount());
  const fetchVocabulary = useDisplayVocabularyStore((state) => state.fetchVocabulary);
  const fetchDefaults = useWizardDefaultsStore((state) => state.fetchDefaults);

  useProjectListSync();
  useNotificationIngestion();
  useNativeNotificationPermission();
  useEffect(() => {
    void fetchVocabulary();
  }, [fetchVocabulary]);
  useEffect(() => {
    void fetchDefaults().then(() => {
      const defaults = useWizardDefaultsStore.getState().defaults;
      if (!defaults) return;
      const wizard = useWizardStore.getState();
      wizard.setProjectData({ planAgent: defaults.defaultAgent });
      if (!wizard.configLoaded) {
        wizard.setFullConfig(defaults.placeholderConfig as import("../types/wizard").WizardConfig);
      }
      if (!useAskStore.getState().activeProjectId) {
        useAskStore.getState().setSelectedAgent(defaults.defaultAgent);
      }
    });
  }, [fetchDefaults]);

  return (
    <div
      className={cn(
        "flex h-full flex-col overflow-hidden border border-border bg-void font-mono text-text",
        isMac ? "rounded-2xl" : "rounded-lg",
      )}
    >
      <TitleBar />
      <SidebarProvider collapsed={isSidebarCollapsed} onCollapsedChange={setIsSidebarCollapsed}>
        <SidebarLayout>
          <AppSidebar
            totalUnread={totalUnread}
            onToggleNotifications={() => setIsNotificationPanelOpen((openState) => !openState)}
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
