import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import {
  currentWindowMock,
  getCurrentWindowMock,
  mockWindowMaximized,
} from "../test/mocks/desktop";
import { TitleBar } from "./TitleBar";

describe("TitleBar", () => {
  it("routes window controls and menu actions through the mocked Tauri window API", async () => {
    const user = userEvent.setup();

    render(<TitleBar />);

    await waitFor(() => expect(getCurrentWindowMock).toHaveBeenCalledTimes(1));
    expect(currentWindowMock.isMaximized).toHaveBeenCalledTimes(1);

    await user.click(screen.getByLabelText("Maximize window"));
    await waitFor(() => expect(screen.getByLabelText("Restore window")).toBeInTheDocument());
    expect(currentWindowMock.toggleMaximize).toHaveBeenCalledTimes(1);

    await user.click(screen.getByLabelText("Minimize window"));
    await user.click(screen.getByLabelText("Close window"));
    expect(currentWindowMock.minimize).toHaveBeenCalledTimes(1);
    expect(currentWindowMock.close).toHaveBeenCalledTimes(1);

    await user.click(screen.getByLabelText("Open window menu"));
    await user.click(await screen.findByRole("menuitem", { name: "Restore" }));
    expect(currentWindowMock.toggleMaximize).toHaveBeenCalledTimes(2);
  });

  it("starts dragging only from draggable titlebar regions", async () => {
    mockWindowMaximized(true);
    render(<TitleBar />);

    await waitFor(() => expect(currentWindowMock.isMaximized).toHaveBeenCalled());
    currentWindowMock.startDragging.mockClear();

    fireEvent.mouseDown(screen.getByText("LoopForge"), { button: 0 });
    fireEvent.mouseDown(screen.getByLabelText("Open window menu"), { button: 0 });
    fireEvent.mouseDown(screen.getByText("LoopForge"), { button: 2 });

    expect(currentWindowMock.startDragging).toHaveBeenCalledTimes(1);
  });
});
