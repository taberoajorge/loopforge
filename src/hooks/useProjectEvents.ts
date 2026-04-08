import { useEffect, useState } from "react";
import {
  onAgentSwitched,
  onIterationCompleted,
  onIterationStarted,
  onRateLimitDetected,
  onSessionEnded,
  onSessionStarted,
  type LoopEvent,
} from "../lib/tauri";

function isSameProject(projectId: string, payload: unknown): boolean {
  if (!payload || typeof payload !== "object") {
    return false;
  }
  const value = (payload as Record<string, unknown>).projectId;
  return typeof value === "string" && value === projectId;
}

export function useProjectEvents(projectId: string | undefined) {
  const [events, setEvents] = useState<LoopEvent[]>([]);

  useEffect(() => {
    if (!projectId) {
      setEvents([]);
      return;
    }

    const listeners: Array<Promise<() => void>> = [];

    listeners.push(
      onSessionStarted((payload: unknown) => {
        if (!isSameProject(projectId, payload)) {
          return;
        }
        setEvents((current) => [...current, { type: "session_started", payload }]);
      }),
    );

    listeners.push(
      onIterationStarted((payload: unknown) => {
        if (!isSameProject(projectId, payload)) {
          return;
        }
        setEvents((current) => [
          ...current,
          { type: "iteration_started", payload },
        ]);
      }),
    );

    listeners.push(
      onIterationCompleted((payload: unknown) => {
        if (!isSameProject(projectId, payload)) {
          return;
        }
        setEvents((current) => [
          ...current,
          { type: "iteration_completed", payload },
        ]);
      }),
    );

    listeners.push(
      onRateLimitDetected((payload: unknown) => {
        if (!isSameProject(projectId, payload)) {
          return;
        }
        setEvents((current) => [
          ...current,
          { type: "rate_limit_detected", payload },
        ]);
      }),
    );

    listeners.push(
      onAgentSwitched((payload: unknown) => {
        if (!isSameProject(projectId, payload)) {
          return;
        }
        setEvents((current) => [
          ...current,
          { type: "agent_switched", payload },
        ]);
      }),
    );

    listeners.push(
      onSessionEnded((payload: unknown) => {
        if (!isSameProject(projectId, payload)) {
          return;
        }
        setEvents((current) => [
          ...current,
          { type: "session_ended", payload },
        ]);
      }),
    );

    return () => {
      listeners.forEach((pending) => {
        pending.then((unlisten) => unlisten());
      });
    };
  }, [projectId]);

  return {
    events,
    clear: () => setEvents([]),
  };
}
