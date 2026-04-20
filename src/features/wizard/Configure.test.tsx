import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useParams } from "react-router";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useAgentStore } from "../../stores/agentStore";
import { useWizardStore } from "../../stores/wizardStore";
import {
  createAgentCapabilities,
  createAgentInfo,
  createProjectConfig,
  createWizardConfig,
  createWizardProjectData,
} from "../../test/fixtures";
import { invokeMock, mockTauriCommands, type TauriCommandArgs } from "../../test/mocks";
import { renderRoute } from "../../test/renderRoute";
import { Configure } from "./Configure";

function resetConfigureStores() {
  useAgentStore.setState({ agents: [], detecting: false });
  useWizardStore.getState().reset();
  useWizardStore.setState({
    projectId: "project-010",
    projectData: createWizardProjectData(),
    stories: [],
    configLoaded: true,
    config: createWizardConfig({
      executeAgent: "codex",
      executeModel: "gpt-5.4",
      executeEffort: "medium",
      fallbackChain: ["claude", "codex"],
      gutterThreshold: 8,
      maxIterations: 80,
      cooldownSeconds: 15,
      testCommand: "bun run verify",
      maxVerificationRetries: 4,
      scmProvider: "github",
      reviewPollingInterval: 90,
      reviewTimeout: 1200,
    }),
  });
}

function LaunchRouteProbe() {
  const params = useParams();
  return <div data-testid="launch-route">{params.id}</div>;
}

describe("Configure", () => {
  beforeEach(() => {
    resetConfigureStores();
  });

  it("loads existing config values and persists edits before advancing", async () => {
    const user = userEvent.setup();
    let savedConfigArgs: TauriCommandArgs<"complete_configure_step"> | undefined;
    mockTauriCommands({
      detect_agents: [createAgentInfo()],
      resolve_agent_selection: {
        resolvedModel: "gpt-5.4",
        resolvedEffort: "medium",
        capabilities: createAgentCapabilities(),
      },
      complete_configure_step: vi.fn((args: TauriCommandArgs<"complete_configure_step">) => {
        savedConfigArgs = args;
        return {
          errors: {},
          nextRoute: `/new/launch/${args.projectId}`,
          config: createProjectConfig({
            executeAgent: args.raw.executeAgent,
            executeModel: args.raw.executeModel,
            executeEffort: args.raw.executeEffort,
            fallbackChain: args.raw.fallbackChain,
            gutterThreshold: args.raw.gutterThreshold,
            maxIterations: args.raw.maxIterations,
            cooldownSeconds: args.raw.cooldownSeconds,
            testCommand: args.raw.testCommand,
            maxVerificationRetries: args.raw.maxVerificationRetries,
            scmProvider: args.raw.scmProvider,
            reviewPollingInterval: args.raw.reviewPollingInterval,
            reviewTimeout: args.raw.reviewTimeout,
          }),
        };
      }),
    });

    renderRoute(
      [
        { path: "/new/configure/:id", element: <Configure /> },
        { path: "/new/launch/:id", element: <LaunchRouteProbe /> },
      ],
      ["/new/configure/project-010"],
    );

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("detect_agents");
      expect(invokeMock).toHaveBeenCalledWith("resolve_agent_selection", {
        agent: "codex",
        currentModel: "gpt-5.4",
        currentEffort: "medium",
      });
    });

    expect(screen.getByLabelText("Gutter threshold")).toHaveValue(8);
    expect(screen.getByLabelText("Max iterations")).toHaveValue(80);
    expect(screen.getByLabelText("Cooldown (s)")).toHaveValue(15);
    expect(screen.getByLabelText("Verification retries")).toHaveValue(4);
    expect(screen.getByLabelText("Test command")).toHaveValue("bun run verify");
    expect(screen.getByLabelText("Poll interval (s)")).toHaveValue(90);
    expect(screen.getByLabelText("Timeout (s)")).toHaveValue(1200);

    await user.clear(screen.getByLabelText("Max iterations"));
    await user.type(screen.getByLabelText("Max iterations"), "120");
    await user.clear(screen.getByLabelText("Test command"));
    await user.type(screen.getByLabelText("Test command"), "bun run test -- --run");
    await user.clear(screen.getByLabelText("Timeout (s)"));
    await user.type(screen.getByLabelText("Timeout (s)"), "1800");
    await user.click(screen.getByRole("button", { name: "Next" }));

    expect(await screen.findByTestId("launch-route")).toHaveTextContent("project-010");
    expect(savedConfigArgs).toBeDefined();

    expect((savedConfigArgs as TauriCommandArgs<"complete_configure_step">).raw).toMatchObject({
      executeAgent: "codex",
      executeModel: "gpt-5.4",
      executeEffort: "medium",
      fallbackChain: ["claude", "codex"],
      maxIterations: 120,
      testCommand: "bun run test -- --run",
      reviewTimeout: 1800,
    });
    expect(useWizardStore.getState().config.maxIterations).toBe(120);
  });

  it("blocks saving and navigation when numeric config is invalid", async () => {
    const user = userEvent.setup();
    mockTauriCommands({
      detect_agents: [createAgentInfo()],
      resolve_agent_selection: {
        resolvedModel: "gpt-5.4",
        resolvedEffort: "medium",
        capabilities: createAgentCapabilities(),
      },
      complete_configure_step: {
        errors: { maxIterations: "Max iterations must be between 1 and 500" },
        nextRoute: "/new/launch/project-010",
        config: createProjectConfig(),
      },
    });

    renderRoute(
      [
        { path: "/new/configure/:id", element: <Configure /> },
        { path: "/new/launch/:id", element: <LaunchRouteProbe /> },
      ],
      ["/new/configure/project-010"],
    );

    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("detect_agents"));

    await user.clear(screen.getByLabelText("Max iterations"));
    await user.type(screen.getByLabelText("Max iterations"), "0");
    await user.click(screen.getByRole("button", { name: "Next" }));

    expect(await screen.findByText("Max iterations must be between 1 and 500")).toBeInTheDocument();
    expect(screen.queryByTestId("launch-route")).not.toBeInTheDocument();
    expect(useWizardStore.getState().currentStep).toBe(1);
  });
});
