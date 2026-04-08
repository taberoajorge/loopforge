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

const EVENT_MAP: Array<{
  event: string;
  type: NotificationType;
  title: (p: EventPayload) => string;
  message: (p: EventPayload) => string;
  filter?: (p: EventPayload) => boolean;
}> = [
  {
    event: "loop:iteration-completed",
    type: "story_completed",
    title: () => "Story completed",
    message: (p) => `Story ${p.storyId ?? "unknown"} passed verification.`,
    filter: (p) => {
      const r = p as Record<string, unknown>;
      return r.result === "success" || r.outcome === "passed";
    },
  },
  {
    event: "loop:story-skipped",
    type: "story_blocked",
    title: () => "Story skipped",
    message: (p) =>
      `${p.storyId ?? "Story"}: ${p.reason ?? "exceeded failure threshold"}`,
  },
  {
    event: "loop:verification-failed",
    type: "loop_error",
    title: (p) =>
      p.circuitBreaker ? "Circuit breaker triggered" : "Verification failed",
    message: (p) =>
      p.circuitBreaker
        ? `${p.storyId ?? "Story"}: same error repeated — skipping`
        : `${p.storyId ?? "Story"}: attempt ${p.attempt ?? "?"} failed (${p.errorCount ?? 0} errors)`,
    filter: (p) => p.circuitBreaker === true || (p.attempt ?? 0) >= 3,
  },
  {
    event: "loop:rate-limit-detected",
    type: "rate_limited",
    title: () => "Rate limited",
    message: (p) => `Agent ${p.agent ?? ""} hit rate limit — switching.`,
  },
  {
    event: "loop:session-ended",
    type: "loop_completed",
    title: () => "Loop finished",
    message: () => "Session completed.",
  },
];

export function useNotificationIngestion() {
  const addNotification = useNotificationStore((s) => s.addNotification);

  useEffect(() => {
    const unlisteners: Array<Promise<() => void>> = [];

    for (const entry of EVENT_MAP) {
      unlisteners.push(
        listen<EventPayload>(entry.event, (event) => {
          const payload = event.payload;
          if (!payload?.projectId) return;
          if (entry.filter && !entry.filter(payload)) return;

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
