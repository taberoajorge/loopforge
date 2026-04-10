import { vi } from "vitest";

const windowState = { maximized: false };

export const currentWindowMock = {
  isMaximized: vi.fn<() => Promise<boolean>>(),
  toggleMaximize: vi.fn<() => Promise<void>>(),
  minimize: vi.fn<() => Promise<void>>(),
  close: vi.fn<() => Promise<void>>(),
  startDragging: vi.fn<() => Promise<void>>(),
};

export const getCurrentWindowMock = vi.fn(() => currentWindowMock);
export const isPermissionGrantedMock = vi.fn<() => Promise<boolean>>();
export const requestPermissionMock = vi.fn<() => Promise<"granted" | "denied">>();
export const openDialogMock = vi.fn<() => Promise<string | string[] | null>>();

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: getCurrentWindowMock,
}));

vi.mock("@tauri-apps/plugin-notification", () => ({
  isPermissionGranted: isPermissionGrantedMock,
  requestPermission: requestPermissionMock,
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: openDialogMock,
}));

export function mockWindowMaximized(maximized: boolean) {
  windowState.maximized = maximized;
}

export function mockNotificationPermission(granted: boolean) {
  isPermissionGrantedMock.mockResolvedValue(granted);
  requestPermissionMock.mockResolvedValue(granted ? "granted" : "denied");
}

export function mockDialogSelection(selection: string | string[] | null) {
  openDialogMock.mockResolvedValue(selection);
}

export function resetDesktopMocks() {
  windowState.maximized = false;
  getCurrentWindowMock.mockClear();
  currentWindowMock.isMaximized.mockReset();
  currentWindowMock.toggleMaximize.mockReset();
  currentWindowMock.minimize.mockReset();
  currentWindowMock.close.mockReset();
  currentWindowMock.startDragging.mockReset();
  currentWindowMock.isMaximized.mockImplementation(async () => windowState.maximized);
  currentWindowMock.toggleMaximize.mockImplementation(async () => {
    windowState.maximized = !windowState.maximized;
  });
  currentWindowMock.minimize.mockResolvedValue();
  currentWindowMock.close.mockResolvedValue();
  currentWindowMock.startDragging.mockResolvedValue();
  mockNotificationPermission(true);
  isPermissionGrantedMock.mockClear();
  requestPermissionMock.mockClear();
  openDialogMock.mockReset();
  openDialogMock.mockResolvedValue(null);
}

resetDesktopMocks();
