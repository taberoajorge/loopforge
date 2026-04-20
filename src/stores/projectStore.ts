import { create } from "zustand";
import { type GroupedProjects, listProjectsGrouped, type ProjectSnapshot } from "../lib/tauri";
import type { Project } from "../types/project";

export type { Project };

interface ProjectState {
  projects: Project[];
  grouped: GroupedProjects;
  loading: boolean;
  setProjects: (projects: Project[]) => void;
  setLoading: (loading: boolean) => void;
  fetchProjects: () => Promise<void>;
  applySnapshot: (snapshot: ProjectSnapshot) => void;
}

const EMPTY_GROUPED: GroupedProjects = { active: [], drafts: [], finished: [], archived: [] };

function regroup(projects: Project[]): GroupedProjects {
  const grouped: GroupedProjects = { active: [], drafts: [], finished: [], archived: [] };
  for (const project of projects) {
    if (project.status === "draft") {
      grouped.drafts.push(project);
    } else if (project.status === "archived") {
      grouped.archived.push(project);
    } else if (project.status === "completed" || project.status === "failed") {
      grouped.finished.push(project);
    } else {
      grouped.active.push(project);
    }
  }
  return grouped;
}

function toFrontendStatus(status: ProjectSnapshot["status"]): Project["status"] {
  if (status === "running") return "active";
  if (status === "ready") return "draft";
  return status;
}

function snapshotToProject(snapshot: ProjectSnapshot): Project {
  return {
    id: snapshot.project.id,
    name: snapshot.project.name,
    description: snapshot.project.description,
    status: toFrontendStatus(snapshot.status),
    workingDirectory: snapshot.project.workingDirectory,
    createdAt: snapshot.project.createdAt,
    updatedAt: snapshot.project.updatedAt,
    storiesCompleted: snapshot.progress.storiesDone,
    totalStories: snapshot.progress.storiesTotal,
    currentAgent: snapshot.config?.executeAgent ?? null,
    sessionStartedAt: snapshot.activeSession?.startedAt ?? null,
    sessionEndedAt: snapshot.activeSession?.endedAt ?? null,
    durationLabel: snapshot.uptimeLabel || null,
    uptimeLabel: snapshot.uptimeLabel || null,
    wizardStep: snapshot.project.wizardStep ?? null,
  };
}

export const useProjectStore = create<ProjectState>()((set, get) => ({
  projects: [],
  grouped: EMPTY_GROUPED,
  loading: false,
  setProjects: (projects) => set({ projects }),
  setLoading: (loading) => set({ loading }),
  fetchProjects: async () => {
    set({ loading: true });
    try {
      const grouped = await listProjectsGrouped();
      const allProjects = [
        ...grouped.active,
        ...grouped.drafts,
        ...grouped.finished,
        ...grouped.archived,
      ];
      set({ projects: allProjects, grouped, loading: false });
    } catch {
      set({ loading: false });
    }
  },
  applySnapshot: (snapshot) => {
    const project = snapshotToProject(snapshot);
    const existing = get().projects;
    const index = existing.findIndex((proj) => proj.id === project.id);
    const projects = [...existing];
    if (index >= 0) {
      projects[index] = project;
    } else {
      projects.push(project);
    }
    set({ projects, grouped: regroup(projects) });
  },
}));
