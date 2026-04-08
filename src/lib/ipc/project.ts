import { invoke } from "@tauri-apps/api/core";
import type { Project } from "../../types/project";
import type {
  IterationStory, ProjectConfig, ProjectRecord, ProjectsByStatus,
  ProjectSnapshot, WizardResumeState,
} from "./types";

export async function listProjects(): Promise<ProjectsByStatus> {
  return invoke<ProjectsByStatus>("list_projects");
}

export async function listProjectsEnriched(): Promise<Project[]> {
  return invoke<Project[]>("list_projects_enriched");
}

export async function createProject(
  name: string, description: string, workingDirectory: string, wizardStep?: string,
): Promise<ProjectRecord> {
  return invoke<ProjectRecord>("create_project", { name, description, workingDirectory, wizardStep });
}

export async function getProjectSnapshot(projectId: string): Promise<ProjectSnapshot> {
  return invoke<ProjectSnapshot>("get_project_snapshot", { projectId });
}

export async function pauseProject(projectId: string): Promise<void> {
  return invoke("pause_project", { projectId });
}

export async function resumeProject(projectId: string): Promise<void> {
  return invoke("resume_project", { projectId });
}

export async function getProjectStories(projectId: string): Promise<IterationStory[]> {
  return invoke<IterationStory[]>("get_project_stories", { projectId });
}

export async function getProjectConfig(projectId: string): Promise<ProjectConfig | null> {
  return invoke<ProjectConfig | null>("get_project_config", { projectId });
}

export async function saveWizardState(
  projectId: string, wizardStep: string, wizardStateJson: string,
): Promise<void> {
  return invoke("save_wizard_state", { projectId, wizardStep, wizardStateJson });
}

export async function saveDraft(projectId: string, draftJson: string): Promise<void> {
  return invoke("save_draft", { projectId, draftJson });
}

export async function loadDraft(projectId: string): Promise<string | null> {
  return invoke<string | null>("load_draft", { projectId });
}

export async function resumeWizard(projectId: string): Promise<WizardResumeState> {
  return invoke<WizardResumeState>("resume_wizard", { projectId });
}

export async function finalizeDraft(projectId: string): Promise<void> {
  return invoke("finalize_draft", { projectId });
}

export async function discardDraft(projectId: string): Promise<void> {
  return invoke("discard_draft", { projectId });
}

export async function archiveProject(projectId: string): Promise<void> {
  return invoke("archive_project", { projectId });
}
