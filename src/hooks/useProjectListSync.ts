import { useEffect } from "react";
import { onProjectStateChanged } from "../lib/tauri";
import { useProjectStore } from "../stores/projectStore";

export function useProjectListSync() {
  const fetchProjects = useProjectStore((state) => state.fetchProjects);
  const applySnapshot = useProjectStore((state) => state.applySnapshot);

  useEffect(() => {
    void fetchProjects();
    const listeners = [
      onProjectStateChanged((payload) => {
        if (payload.snapshot) {
          applySnapshot(payload.snapshot);
          return;
        }
        void fetchProjects();
      }),
    ];

    return () => {
      listeners.forEach((pending) => {
        pending.then((unlisten) => unlisten());
      });
    };
  }, [applySnapshot, fetchProjects]);
}
