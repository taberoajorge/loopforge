import { create } from "zustand";
import { type DisplayVocabulary, getDisplayVocabulary } from "../lib/ipc/display-vocabulary";

interface DisplayVocabularyState {
  vocabulary: DisplayVocabulary | null;
  loading: boolean;
  fetchVocabulary: () => Promise<void>;
}

export const useDisplayVocabularyStore = create<DisplayVocabularyState>()((set) => ({
  vocabulary: null,
  loading: false,
  fetchVocabulary: async () => {
    set({ loading: true });
    try {
      const vocabulary = await getDisplayVocabulary();
      set({ vocabulary });
    } catch {
      set({ vocabulary: null });
    }
    set({ loading: false });
  },
}));
