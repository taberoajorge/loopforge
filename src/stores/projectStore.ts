import { create } from "zustand";
import { listProjectsEnriched } from "../lib/tauri";
import type { Project } from "../types/project";

export type { Project };

interface ProjectState {
  projects: Project[];
  loading: boolean;
  setProjects: (projects: Project[]) => void;
  setLoading: (loading: boolean) => void;
  fetchProjects: () => Promise<void>;
  upsertProject: (project: Project) => void;
}

export const useProjectStore = create<ProjectState>()((set, get) => ({
  projects: [],
  loading: false,
  setProjects: (projects) => set({ projects }),
  setLoading: (loading) => set({ loading }),
  fetchProjects: async () => {
    set({ loading: true });
    try {
      const projects = await listProjectsEnriched();
      set({ projects, loading: false });
    } catch {
      set({ loading: false });
    }
  },
  upsertProject: (project) => {
    const existing = get().projects;
    const index = existing.findIndex((proj) => proj.id === project.id);
    if (index >= 0) {
      const updated = [...existing];
      updated[index] = project;
      set({ projects: updated });
    } else {
      set({ projects: [...existing, project] });
    }
  },
}));
