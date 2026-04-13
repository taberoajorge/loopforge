import { create } from "zustand";
import { getWizardDefaults, type WizardDefaultsResponse } from "../lib/tauri";

interface WizardDefaultsState {
  defaults: WizardDefaultsResponse | null;
  loading: boolean;
  fetchDefaults: () => Promise<void>;
}

export const useWizardDefaultsStore = create<WizardDefaultsState>()((set) => ({
  defaults: null,
  loading: false,
  fetchDefaults: async () => {
    set({ loading: true });
    try {
      const defaults = await getWizardDefaults();
      set({ defaults });
    } catch {
      set({ defaults: null });
    }
    set({ loading: false });
  },
}));
