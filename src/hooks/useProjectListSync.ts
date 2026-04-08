import { useEffect } from "react";
import { onIterationCompleted, onSessionEnded, onSessionStarted } from "../lib/tauri";
import { useProjectStore } from "../stores/projectStore";

export function useProjectListSync() {
  const fetchProjects = useProjectStore((state) => state.fetchProjects);

  useEffect(() => {
    void fetchProjects();

    const refreshTimer = window.setInterval(() => {
      void fetchProjects();
    }, 5000);

    const listeners = [
      onSessionStarted(() => {
        void fetchProjects();
      }),
      onIterationCompleted(() => {
        void fetchProjects();
      }),
      onSessionEnded(() => {
        void fetchProjects();
      }),
    ];

    return () => {
      window.clearInterval(refreshTimer);
      listeners.forEach((pending) => {
        pending.then((unlisten) => unlisten());
      });
    };
  }, [fetchProjects]);
}
