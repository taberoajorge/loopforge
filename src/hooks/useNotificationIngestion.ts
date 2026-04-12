import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  useNotificationStore,
  type NotificationType,
} from "../stores/notificationStore";

interface EventPayload {
  projectId?: string;
  storyId?: string;
  attempt?: number;
  errorCount?: number;
  circuitBreaker?: boolean;
  reason?: string;
  agent?: string;
  sessionId?: string;
}

interface NotificationEntry {
  event: string;
  type: NotificationType;
  title: (payload: EventPayload) => string;
  message: (payload: EventPayload) => string;
}

const EVENT_MAP: NotificationEntry[] = [
  {
    event: "loop:iteration-completed",
    type: "story_completed",
    title: () => "Story completed",
    message: (payload) => `Story ${payload.storyId ?? "unknown"} passed verification.`,
  },
  {
    event: "loop:story-skipped",
    type: "story_blocked",
    title: () => "Story skipped",
    message: (payload) =>
      `${payload.storyId ?? "Story"}: ${payload.reason ?? "exceeded failure threshold"}`,
  },
  {
    event: "loop:verification-failed",
    type: "loop_error",
    title: (payload) =>
      payload.circuitBreaker ? "Circuit breaker triggered" : "Verification failed",
    message: (payload) =>
      payload.circuitBreaker
        ? `${payload.storyId ?? "Story"}: same error repeated`
        : `${payload.storyId ?? "Story"}: attempt ${payload.attempt ?? "?"} failed (${payload.errorCount ?? 0} errors)`,
  },
  {
    event: "loop:rate-limit-detected",
    type: "rate_limited",
    title: () => "Rate limited",
    message: (payload) => `Agent ${payload.agent ?? ""} hit rate limit.`,
  },
  {
    event: "loop:session-ended",
    type: "loop_completed",
    title: () => "Loop finished",
    message: () => "Session completed.",
  },
];

export function useNotificationIngestion() {
  const addNotification = useNotificationStore((state) => state.addNotification);

  useEffect(() => {
    const unlisteners: Array<Promise<() => void>> = [];

    for (const entry of EVENT_MAP) {
      unlisteners.push(
        listen<EventPayload>(entry.event, (event) => {
          const payload = event.payload;
          if (!payload?.projectId) return;

          addNotification({
            projectId: payload.projectId,
            type: entry.type,
            title: entry.title(payload),
            message: entry.message(payload),
          });
        }),
      );
    }

    return () => {
      for (const pending of unlisteners) {
        pending.then((unlisten) => unlisten());
      }
    };
  }, [addNotification]);
}
