import { useEffect, useRef, useState } from "react";
import { reportError } from "../lib/reportError";
import {
  onPlanActivityBatch,
  onPlanComplete,
  onPlanError,
  onPlanHeartbeat,
  queryPlanStatus,
} from "../lib/tauri";
import { type PlanEvent, useWizardStore } from "../stores/wizardStore";

export function usePlanEvents(projectId: string | undefined) {
  const [planError, setPlanError] = useState<string | null>(null);
  const mountIdRef = useRef(0);

  useEffect(() => {
    if (!projectId) return;

    const currentMountId = ++mountIdRef.current;
    const unlistenFns: Array<() => void> = [];
    let cleaned = false;

    function isCancelled() {
      return cleaned || mountIdRef.current !== currentMountId;
    }

    queryPlanStatus(projectId)
      .then((info) => {
        if (isCancelled()) return;
        if (info && info.status === "running") {
          useWizardStore.getState().setPlanRunning(true);
        }
      })
      .catch((caughtError: unknown) => {
        reportError("usePlanEvents.queryPlanStatus", caughtError);
      });

    onPlanActivityBatch((payload) => {
      if (isCancelled() || payload.projectId !== projectId) return;

      const current = useWizardStore.getState();

      if (payload.events.length > 0) {
        const events: PlanEvent[] = payload.events.map((raw) => ({
          kind: raw.kind,
          content: raw.content,
          timestamp: raw.timestamp,
        }));
        current.appendPlanEvents(events);
      }
      if (payload.planContent !== undefined) {
        current.setPlanContent(payload.planContent);
      }
      if (!current.planRunning) {
        current.setPlanRunning(true);
      }
    }).then((unlisten) => {
      if (isCancelled()) {
        unlisten();
        return;
      }
      unlistenFns.push(unlisten);
    });

    onPlanComplete((payload) => {
      if (isCancelled() || payload.projectId !== projectId) return;
      const current = useWizardStore.getState();
      current.setPlanRunning(false);
      setPlanError(null);
      if (payload.finalContent && payload.finalContent.length > 0) {
        useWizardStore.setState({ planContent: payload.finalContent });
      }
      current.setPlanComplete(true);
    }).then((unlisten) => {
      if (isCancelled()) {
        unlisten();
        return;
      }
      unlistenFns.push(unlisten);
    });

    onPlanError((payload) => {
      if (isCancelled() || payload.projectId !== projectId) return;
      useWizardStore.getState().setPlanRunning(false);
      setPlanError(payload.detail || "Agent exited with an error");
    }).then((unlisten) => {
      if (isCancelled()) {
        unlisten();
        return;
      }
      unlistenFns.push(unlisten);
    });

    onPlanHeartbeat((payload) => {
      if (isCancelled() || payload.projectId !== projectId) return;
    }).then((unlisten) => {
      if (isCancelled()) {
        unlisten();
        return;
      }
      unlistenFns.push(unlisten);
    });

    return () => {
      cleaned = true;
      for (const unlisten of unlistenFns) {
        unlisten();
      }
    };
  }, [projectId]);

  return { planError, clearError: () => setPlanError(null) };
}
