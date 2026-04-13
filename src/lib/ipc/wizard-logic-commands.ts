import { invoke } from "@tauri-apps/api/core";
import type {
  AdvanceWizardResult,
  CompleteConfigureResult,
  CompleteDescribeInput,
  ConfigDefaultsResponse,
  LaunchProjectResult,
  LaunchReadiness,
  RawConfigInput,
  StoriesResponse,
  SubmitConfigResult,
  ValidationErrors,
  WizardDefaultsResponse,
  WizardHydrationResult,
  WizardRouteResult,
} from "./wizard-logic-types";

export async function hydrateWizard(projectId: string): Promise<WizardHydrationResult> {
  return invoke<WizardHydrationResult>("hydrate_wizard", { projectId });
}

export async function advanceWizardStep(targetStep: number): Promise<AdvanceWizardResult> {
  return invoke<AdvanceWizardResult>("advance_wizard_step", { targetStep });
}

export async function getDefaultConfig(): Promise<ConfigDefaultsResponse> {
  return invoke<ConfigDefaultsResponse>("get_default_config");
}

export async function validateProjectConfig(configJson: string): Promise<ValidationErrors> {
  return invoke<ValidationErrors>("validate_project_config", { configJson });
}

export async function validateDescribeInput(args: {
  name: string;
  description: string;
  workingDirectory: string;
  planAgent: string;
}): Promise<ValidationErrors> {
  return invoke<ValidationErrors>("validate_describe_input", args);
}

export async function validateLaunchReadiness(projectId: string): Promise<LaunchReadiness> {
  return invoke<LaunchReadiness>("validate_launch_readiness", { projectId });
}

export async function addStory(projectId: string): Promise<StoriesResponse> {
  return invoke<StoriesResponse>("add_story", { projectId });
}

export async function updateStoryBackend(
  projectId: string,
  storyId: string,
  patchJson: string,
): Promise<StoriesResponse> {
  return invoke<StoriesResponse>("update_story", { projectId, storyId, patchJson });
}

export async function removeStoryBackend(
  projectId: string,
  storyId: string,
): Promise<StoriesResponse> {
  return invoke<StoriesResponse>("remove_story", { projectId, storyId });
}

export async function reorderStoriesBackend(
  projectId: string,
  fromIndex: number,
  toIndex: number,
): Promise<StoriesResponse> {
  return invoke<StoriesResponse>("reorder_stories", { projectId, fromIndex, toIndex });
}

export async function getStories(projectId: string): Promise<StoriesResponse> {
  return invoke<StoriesResponse>("get_stories", { projectId });
}

export async function saveWizardDraft(projectId: string, stepName: string): Promise<void> {
  return invoke("save_wizard_draft", { projectId, stepName });
}

export async function replan(projectId: string, feedback: string): Promise<void> {
  return invoke("replan", { projectId, feedback });
}

export async function submitProjectConfig(
  projectId: string,
  raw: RawConfigInput,
): Promise<SubmitConfigResult> {
  return invoke<SubmitConfigResult>("submit_project_config", { projectId, raw });
}

export async function completeDescribeStep(
  input: CompleteDescribeInput,
): Promise<WizardRouteResult> {
  return invoke<WizardRouteResult>("complete_describe_step", { input });
}

export async function completeAtomizeStep(projectId: string): Promise<WizardRouteResult> {
  return invoke<WizardRouteResult>("complete_atomize_step", { projectId });
}

export async function completeConfigureStep(
  projectId: string,
  raw: RawConfigInput,
): Promise<CompleteConfigureResult> {
  return invoke<CompleteConfigureResult>("complete_configure_step", { projectId, raw });
}

export async function launchProject(projectId: string): Promise<LaunchProjectResult> {
  return invoke<LaunchProjectResult>("launch_project", { projectId });
}

export async function exitWizard(projectId: string, currentStep: number): Promise<void> {
  return invoke("exit_wizard", { projectId, currentStep });
}

export async function getWizardDefaults(): Promise<WizardDefaultsResponse> {
  return invoke<WizardDefaultsResponse>("get_wizard_defaults");
}

export async function markWizardStale(projectId: string, fromStep: number): Promise<void> {
  return invoke("mark_wizard_stale", { projectId, fromStep });
}
