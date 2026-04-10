import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach, vi } from "vitest";
import { resetDesktopMocks, resetTauriMocks } from "./mocks";
import "./mocks/desktop";

const storage = new Map<string, string>();
const localStorageMock = {
  getItem: (key: string) => storage.get(key) ?? null,
  setItem: (key: string, value: string) => {
    storage.set(key, value);
  },
  removeItem: (key: string) => {
    storage.delete(key);
  },
  clear: () => {
    storage.clear();
  },
};

Object.defineProperty(window, "localStorage", {
  value: localStorageMock,
  configurable: true,
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  localStorageMock.clear();
  resetDesktopMocks();
  resetTauriMocks();
});
