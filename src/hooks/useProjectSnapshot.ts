import { useCallback, useEffect, useState } from "react";
import { getProjectSnapshot, type ProjectSnapshot } from "../lib/tauri";

interface SnapshotState {
  snapshot: ProjectSnapshot | null;
  loading: boolean;
  error: string | null;
}

export function useProjectSnapshot(projectId: string | undefined, pollMs = 5000) {
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
    if (!projectId || pollMs <= 0) {
      return;
    }
    const timer = window.setInterval(() => {
      refresh();
    }, pollMs);
    return () => window.clearInterval(timer);
  }, [pollMs, projectId, refresh]);

  return {
    snapshot: state.snapshot,
    loading: state.loading,
    error: state.error,
    refresh,
  };
}
