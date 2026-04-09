import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { createProjectScopedEventPayload } from "../test/fixtures";
import { emitTauriEvent, getEventListenerCount } from "../test/mocks";
import { useProjectEvents } from "./useProjectEvents";

describe("useProjectEvents", () => {
  it("collects matching project events, clears state, and unsubscribes listeners", async () => {
    const { result, unmount } = renderHook(() => useProjectEvents("project-001"));

    await waitFor(() => expect(getEventListenerCount("loop:session-started")).toBe(1));
    expect(getEventListenerCount("loop:iteration-started")).toBe(1);
    expect(getEventListenerCount("loop:iteration-completed")).toBe(1);
    expect(getEventListenerCount("loop:rate-limit-detected")).toBe(1);
    expect(getEventListenerCount("loop:agent-switched")).toBe(1);
    expect(getEventListenerCount("loop:session-ended")).toBe(1);

    await act(async () => {
      await emitTauriEvent("loop:session-started", createProjectScopedEventPayload());
      await emitTauriEvent(
        "loop:iteration-started",
        createProjectScopedEventPayload({ projectId: "project-999", storyId: "S-999" }),
      );
      await emitTauriEvent(
        "loop:iteration-completed",
        createProjectScopedEventPayload({ storyId: "S-006", result: "success" }),
      );
      await emitTauriEvent(
        "loop:rate-limit-detected",
        createProjectScopedEventPayload({ agent: "codex" }),
      );
    });

    expect(result.current.events).toEqual([
      { type: "session_started", payload: expect.objectContaining({ projectId: "project-001" }) },
      { type: "iteration_completed", payload: expect.objectContaining({ storyId: "S-006" }) },
      { type: "rate_limit_detected", payload: expect.objectContaining({ agent: "codex" }) },
    ]);

    act(() => {
      result.current.clear();
    });
    expect(result.current.events).toEqual([]);

    unmount();

    await waitFor(() => expect(getEventListenerCount("loop:session-started")).toBe(0));
    expect(getEventListenerCount("loop:iteration-started")).toBe(0);
    expect(getEventListenerCount("loop:iteration-completed")).toBe(0);
    expect(getEventListenerCount("loop:rate-limit-detected")).toBe(0);
    expect(getEventListenerCount("loop:agent-switched")).toBe(0);
    expect(getEventListenerCount("loop:session-ended")).toBe(0);
  });

  it("resets event state when no project id is provided", () => {
    const { result, rerender } = renderHook(
      ({ projectId }: { projectId?: string }) => useProjectEvents(projectId),
      { initialProps: { projectId: "project-001" as string | undefined } },
    );

    act(() => {
      rerender({ projectId: undefined });
    });

    expect(result.current.events).toEqual([]);
  });
});
