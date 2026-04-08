import { useEffect, useRef, useState } from "react";
import {
  onPlanActivityBatch, onPlanComplete, onPlanError, onPlanHeartbeat,
  queryPlanStatus, loadExistingPlan,
  type PlanActivityBatchPayload,
} from "../lib/tauri";
import { useWizardStore, type PlanEvent } from "../stores/wizardStore";
import {
  isNoisePlanLine, normalizePlanLine,
} from "../features/wizard/components/PlanStreamPanel";

function processBatch(
  payload: PlanActivityBatchPayload,
  lastLineRef: React.RefObject<string>,
): PlanEvent[] {
  const filtered: PlanEvent[] = [];
  for (const raw of payload.events) {
    const normalizedContent = normalizePlanLine(raw.content);
    if (isNoisePlanLine(normalizedContent)) continue;
    const signature = `${raw.kind}:${normalizedContent}`;
    if (lastLineRef.current === signature) continue;
    lastLineRef.current = signature;
    filtered.push({
      kind: raw.kind,
      content: normalizedContent,
      timestamp: raw.timestamp,
    });
  }
  return filtered;
}

export function usePlanEvents(projectId: string | undefined) {
  const [planError, setPlanError] = useState<string | null>(null);
  const lastLineRef = useRef("");
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
      .catch(() => {});

    onPlanActivityBatch((payload) => {
      if (isCancelled() || payload.projectId !== projectId) return;

      const filtered = processBatch(payload, lastLineRef);
      const current = useWizardStore.getState();

      if (filtered.length > 0) {
        current.appendPlanEvents(filtered);
      }
      if (payload.planContentDelta) {
        current.appendPlanContentDelta(payload.planContentDelta);
      }
      if (!current.planRunning) {
        current.setPlanRunning(true);
      }
    }).then((unlisten) => {
      if (isCancelled()) { unlisten(); return; }
      unlistenFns.push(unlisten);
    });

    onPlanComplete((payload) => {
      if (isCancelled() || payload.projectId !== projectId) return;
      const current = useWizardStore.getState();
      current.setPlanRunning(false);
      setPlanError(null);
      loadExistingPlan(projectId)
        .then((diskContent) => {
          if (isCancelled()) return;
          const store = useWizardStore.getState();
          if (diskContent && diskContent.length > 0) {
            useWizardStore.setState({ planContent: diskContent });
          }
          store.setPlanComplete(true);
        })
        .catch(() => {
          useWizardStore.getState().setPlanComplete(true);
        });
    }).then((unlisten) => {
      if (isCancelled()) { unlisten(); return; }
      unlistenFns.push(unlisten);
    });

    onPlanError((payload) => {
      if (isCancelled() || payload.projectId !== projectId) return;
      useWizardStore.getState().setPlanRunning(false);
      setPlanError(payload.detail || "Agent exited with an error");
    }).then((unlisten) => {
      if (isCancelled()) { unlisten(); return; }
      unlistenFns.push(unlisten);
    });

    onPlanHeartbeat((payload) => {
      if (isCancelled() || payload.projectId !== projectId) return;
    }).then((unlisten) => {
      if (isCancelled()) { unlisten(); return; }
      unlistenFns.push(unlisten);
    });

    return () => {
      cleaned = true;
      for (const unlisten of unlistenFns) {
        unlisten();
      }
    };
  }, [projectId]);

  useEffect(() => {
    if (!projectId) return;
    const { planRunning } = useWizardStore.getState();
    if (planRunning) return;

    queryPlanStatus(projectId)
      .then((info) => {
        if (!info) {
          loadExistingPlan(projectId)
            .then((content) => {
              if (!content) return;
              const current = useWizardStore.getState();
              if (!current.planContent && content.length > 0) {
                current.appendPlanContent(content);
                current.setPlanComplete(true);
              }
            })
            .catch(() => {});
        }
      })
      .catch(() => {});
  }, [projectId]);

  return { planError, clearError: () => setPlanError(null) };
}
