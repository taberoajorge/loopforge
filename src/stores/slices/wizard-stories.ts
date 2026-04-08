import type { StateCreator } from "zustand";
import type { UserStory } from "../../types/wizard";

export interface WizardStoriesSlice {
  stories: UserStory[];
  setStories: (stories: UserStory[]) => void;
  updateStory: (id: string, patch: Partial<UserStory>) => void;
  reorderStories: (fromIndex: number, toIndex: number) => void;
  addStory: (story: UserStory) => void;
  removeStory: (id: string) => void;
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
  updateStory: (id, patch) =>
    set((state) => ({
      stories: state.stories.map((story) =>
        story.id === id ? { ...story, ...patch } : story,
      ),
    })),
  reorderStories: (fromIndex, toIndex) =>
    set((state) => {
      const reordered = [...state.stories];
      const [moved] = reordered.splice(fromIndex, 1);
      reordered.splice(toIndex, 0, moved);
      return { stories: reordered };
    }),
  addStory: (story) =>
    set((state) => ({ stories: [...state.stories, story] })),
  removeStory: (id) =>
    set((state) => ({
      stories: state.stories.filter((story) => story.id !== id),
    })),
});
