import { useEffect, useState } from "react";
import {
  type AtomizeActivityPayload,
  getAtomizerActivityLog,
  onAtomizationActivity,
} from "../lib/tauri";
import type { AtomizeActivityEvent } from "../types/wizard";

const MAX_ACTIVITY_EVENTS = 300;

function payloadToEvent(payload: AtomizeActivityPayload): AtomizeActivityEvent {
  return { kind: payload.kind, content: payload.content, timestamp: payload.timestamp };
}

export function useAtomizerActivity(projectId: string | undefined) {
  const [events, setEvents] = useState<AtomizeActivityEvent[]>([]);

  useEffect(() => {
    if (!projectId) return;
    setEvents([]);
    let disposed = false;

    getAtomizerActivityLog(projectId).then((backlog) => {
      if (disposed) return;
      setEvents(backlog.map(payloadToEvent));
    });

    const unlistenPromise = onAtomizationActivity((payload: AtomizeActivityPayload) => {
      if (disposed || payload.projectId !== projectId) return;
      setEvents((previous) => {
        const next = [...previous, payloadToEvent(payload)];
        return next.length > MAX_ACTIVITY_EVENTS ? next.slice(-MAX_ACTIVITY_EVENTS) : next;
      });
    });

    return () => {
      disposed = true;
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [projectId]);

  return events;
}
