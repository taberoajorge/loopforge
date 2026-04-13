import { useCallback, useEffect, useState } from "react";
import {
  getProjectSnapshot,
  onProjectStateChanged,
  onStoriesUpdated,
  type ProjectSnapshot,
} from "../lib/tauri";

interface SnapshotState {
  snapshot: ProjectSnapshot | null;
  loading: boolean;
  error: string | null;
}

export function useProjectSnapshot(projectId: string | undefined) {
  const [state, setState] = useState<SnapshotState>({
    snapshot: null,
    loading: true,
    error: null,
  });

  const refresh = useCallback(async () => {
    if (!projectId) {
      setState({ snapshot: null, loading: false, error: null });
      return;
    }
    setState((current) => ({
      ...current,
      loading: current.snapshot === null,
    }));
    try {
      const snapshot = await getProjectSnapshot(projectId);
      setState({ snapshot, loading: false, error: null });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setState((current) => ({
        snapshot: current.snapshot,
        loading: false,
        error: message,
      }));
    }
  }, [projectId]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  useEffect(() => {
    if (!projectId) return;
    const listeners = [
      onProjectStateChanged((payload) => {
        if (payload.projectId !== projectId) return;
        if (payload.snapshot) {
          setState({ snapshot: payload.snapshot, loading: false, error: null });
          return;
        }
        void refresh();
      }),
      onStoriesUpdated((payload) => {
        if (payload.projectId === projectId) void refresh();
      }),
    ];
    return () => {
      listeners.forEach((pending) => {
        pending.then((unlisten) => unlisten());
      });
    };
  }, [projectId, refresh]);

  return {
    snapshot: state.snapshot,
    loading: state.loading,
    error: state.error,
    refresh,
  };
}
