import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach, vi } from "vitest";
import { resetTauriMocks } from "./mocks";

afterEach(() => {
  cleanup();
  resetTauriMocks();
  vi.restoreAllMocks();
});
