import { useCallback, useState } from "react";
import { reportError } from "../../../lib/reportError";
import { pauseProject, resumeProject, stopLoop } from "../../../lib/tauri";

type MonitorAction = "pause" | "resume" | "stop";

interface UseMonitorActionsArgs {
  projectId: string;
  refresh: () => Promise<void>;
  fetchProjects: () => Promise<void>;
}

export function useMonitorActions(args: UseMonitorActionsArgs) {
  const [busyAction, setBusyAction] = useState<MonitorAction | null>(null);

  const runAction = useCallback(
    async (action: MonitorAction) => {
      setBusyAction(action);
      try {
        if (action === "pause") {
          await pauseProject(args.projectId);
        } else if (action === "resume") {
          await resumeProject(args.projectId);
        } else {
          await stopLoop(args.projectId);
        }
        await args.refresh();
        await args.fetchProjects();
      } catch (caughtError: unknown) {
        reportError("useMonitorActions.runAction", caughtError);
      } finally {
        setBusyAction(null);
      }
    },
    [args.fetchProjects, args.projectId, args.refresh],
  );

  return { busyAction, runAction };
}
