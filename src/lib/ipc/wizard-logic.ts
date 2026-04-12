import { invoke } from "@tauri-apps/api/core";
import type { ProjectConfig } from "./types";
import type { UserStory } from "../../types/wizard";

export interface AdvanceWizardResult {
  currentStep: number;
  highestStep: number;
  stepName: string;
}

export interface ConfigLimits {
  gutterThreshold: [number, number];
  maxIterations: [number, number];
  cooldownSeconds: [number, number];
  maxVerificationRetries: [number, number];
  reviewPollingInterval: [number, number];
  reviewTimeout: [number, number];
}

export interface ConfigDefaultsResponse {
  config: ProjectConfig;
  limits: ConfigLimits;
}

export interface ValidationErrors {
  errors: Record<string, string>;
}

export interface LaunchReadiness {
  ready: boolean;
  issues: string[];
}

export interface StoriesResponse {
  stories: UserStory[];
  totalEstimatedMinutes: number;
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

export async function validateDescribeInput(inputJson: string): Promise<ValidationErrors> {
  return invoke<ValidationErrors>("validate_describe_input", { inputJson });
}

export async function validateLaunchReadiness(projectId: string): Promise<LaunchReadiness> {
  return invoke<LaunchReadiness>("validate_launch_readiness", { projectId });
}

export async function addStory(projectId: string): Promise<StoriesResponse> {
  return invoke<StoriesResponse>("add_story", { projectId });
}

export async function updateStoryBackend(
  projectId: string, storyId: string, patchJson: string,
): Promise<StoriesResponse> {
  return invoke<StoriesResponse>("update_story", { projectId, storyId, patchJson });
}

export async function removeStoryBackend(
  projectId: string, storyId: string,
): Promise<StoriesResponse> {
  return invoke<StoriesResponse>("remove_story", { projectId, storyId });
}

export async function reorderStoriesBackend(
  projectId: string, fromIndex: number, toIndex: number,
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

export async function markWizardStale(projectId: string, fromStep: number): Promise<void> {
  return invoke("mark_wizard_stale", { projectId, fromStep });
}
