import { create } from "zustand";
import { getProjectSnapshot, listProjects, type ProjectsByStatus, type ProjectRecord } from "../lib/tauri";

export interface Project {
  id: string;
  name: string;
  description: string;
  status: "draft" | "active" | "paused" | "completed" | "failed" | "blocked" | "archived";
  workingDirectory: string;
  createdAt: string;
  updatedAt: string;
  storiesCompleted?: number;
  totalStories?: number;
  currentAgent?: string | null;
  sessionStartedAt?: string | null;
  sessionEndedAt?: string | null;
  wizardStep?: string | null;
}

function flattenByStatus(grouped: ProjectsByStatus): Project[] {
  const all: ProjectRecord[] = [
    ...grouped.active,
    ...grouped.paused,
    ...grouped.blocked,
    ...grouped.failed,
    ...grouped.completed,
    ...grouped.draft,
    ...grouped.archived,
  ];

  return all.map((record) => ({
    id: record.id,
    name: record.name,
    description: record.description,
    status: record.status as Project["status"],
    workingDirectory: record.workingDirectory,
    createdAt: record.createdAt,
    updatedAt: record.updatedAt,
    wizardStep: (record as unknown as Record<string, unknown>).wizardStep as string | null | undefined,
  }));
}

interface ProjectState {
  projects: Project[];
  loading: boolean;
  setProjects: (projects: Project[]) => void;
  setLoading: (loading: boolean) => void;
  fetchProjects: () => Promise<void>;
  upsertProject: (project: Project) => void;
}

function normalizeStatus(status: string): Project["status"] {
  if (status === "running") return "active";
  if (status === "ready") return "draft";
  if (
    status === "active" ||
    status === "paused" ||
    status === "completed" ||
    status === "failed" ||
    status === "blocked" ||
    status === "draft" ||
    status === "archived"
  ) {
    return status;
  }
  return "draft";
}

async function enrichProject(project: Project): Promise<Project> {
  if (project.status === "draft") return project;
  try {
    const snapshot = await getProjectSnapshot(project.id);
    return {
      ...project,
      status: normalizeStatus(snapshot.status),
      storiesCompleted: snapshot.progress.storiesDone,
      totalStories: snapshot.progress.storiesTotal,
      currentAgent: snapshot.config?.executeAgent ?? null,
      sessionStartedAt: snapshot.activeSession?.startedAt ?? null,
      sessionEndedAt: snapshot.activeSession?.endedAt ?? null,
      updatedAt: snapshot.project.updatedAt,
      wizardStep: snapshot.project.wizardStep ?? project.wizardStep ?? null,
    };
  } catch {
    return project;
  }
}

export const useProjectStore = create<ProjectState>()((set, get) => ({
  projects: [],
  loading: false,
  setProjects: (projects) => set({ projects }),
  setLoading: (loading) => set({ loading }),
  fetchProjects: async () => {
    set({ loading: true });
    try {
      const grouped = await listProjects();
      const projects = flattenByStatus(grouped);
      const enrichedProjects = await Promise.all(
        projects.map((project) => enrichProject(project)),
      );
      set({ projects: enrichedProjects, loading: false });
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
