import type { StateCreator } from "zustand";
import type { UserStory } from "../../types/wizard";

export interface WizardStoriesSlice {
  stories: UserStory[];
  setStories: (stories: UserStory[]) => void;
}

export const STORIES_DEFAULTS = {
  stories: [] as UserStory[],
};

export const createStoriesSlice: StateCreator<
  WizardStoriesSlice,
  [],
  [],
  WizardStoriesSlice
> = (set) => ({
  ...STORIES_DEFAULTS,
  setStories: (stories) => set({ stories }),
});
