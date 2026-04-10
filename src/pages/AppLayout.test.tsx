import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";
import { createProject } from "../test/fixtures";
import {
  mockNotificationPermission,
  mockTauriCommands,
  requestPermissionMock,
} from "../test/mocks";
import { renderRoute } from "../test/renderRoute";
import { useNotificationStore } from "../stores/notificationStore";
import { useProjectStore } from "../stores/projectStore";
import { AppLayout } from "./AppLayout";

function renderAppLayout() {
  return renderRoute(
    [{
      path: "/",
      element: <AppLayout />,
      children: [
        { index: true, element: <div data-testid="home-route">home</div> },
        { path: "monitor/:id", element: <div data-testid="monitor-route">monitor</div> },
      ],
    }],
    ["/"],
  );
}

describe("AppLayout", () => {
  beforeEach(() => {
    useProjectStore.setState({ projects: [], loading: false });
    useNotificationStore.setState({ notifications: [] });
    mockTauriCommands({ list_projects_enriched: [createProject()] });
  });

  it("skips the native permission prompt when notification access is already granted", async () => {
    mockNotificationPermission(true);

    renderAppLayout();

    await waitFor(() => expect(requestPermissionMock).not.toHaveBeenCalled());
    expect(screen.getByTestId("home-route")).toBeInTheDocument();
  });

  it("requests permission and drives shell toggles through mocked desktop seams", async () => {
    const user = userEvent.setup();
    mockNotificationPermission(false);
    useNotificationStore.getState().addNotification({
      projectId: "project-001",
      type: "loop_completed",
      title: "Loop finished",
      message: "Session completed.",
    });

    renderAppLayout();

    await waitFor(() => expect(requestPermissionMock).toHaveBeenCalledTimes(1));

    const sidebar = screen.getByTestId("app-sidebar");
    expect(sidebar).toHaveAttribute("data-state", "expanded");

    await user.click(screen.getByTestId("app-sidebar-toggle-navigation"));
    expect(sidebar).toHaveAttribute("data-state", "collapsed");

    await user.click(screen.getByTestId("app-sidebar-toggle-notifications"));
    const notificationPanel = await screen.findByText("Notifications");
    expect(notificationPanel).toBeInTheDocument();
    expect(screen.getByText("Loop finished")).toBeInTheDocument();

    const unreadButton = screen.getByTestId("app-sidebar-toggle-notifications");
    expect(unreadButton.querySelector("span")).not.toBeNull();
    expect(useNotificationStore.getState().totalUnreadCount()).toBe(1);

    await user.click(screen.getByText("Loop finished"));
    await waitFor(() => expect(screen.getByTestId("monitor-route")).toHaveTextContent("monitor"));
    expect(useNotificationStore.getState().totalUnreadCount()).toBe(0);
  });
});
