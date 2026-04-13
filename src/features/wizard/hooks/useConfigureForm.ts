import { type DragEvent, useEffect, useMemo, useReducer, useRef } from "react";
import { useNavigate } from "react-router";
import { reportError } from "../../../lib/reportError";
import {
  completeConfigureStep,
  detectAgents,
  getDefaultConfig,
  resolveAgentSelection,
} from "../../../lib/tauri";
import type { WizardConfig } from "../../../types/wizard";
import type {
  ConfigureFormHandlers,
  ConfigureFormOptions,
  ScmProvider,
} from "../configureFormTypes";
import type { ConfigureErrors } from "../configureValidation";
import { configureFormReducer, toValues } from "./configureFormReducer";

interface UseConfigureFormArgs {
  projectId: string | undefined;
  agents: Array<{ name: string; installed: boolean; version: string | null }>;
  config: WizardConfig;
  configLoaded: boolean;
  setConfig: (config: WizardConfig) => void;
  setFullConfig: (config: WizardConfig) => void;
  setAgents: (agents: Array<{ name: string; installed: boolean; version: string | null }>) => void;
  setDetecting: (detecting: boolean) => void;
}

export function useConfigureForm(args: UseConfigureFormArgs) {
  const navigate = useNavigate();
  const dragIndexRef = useRef<number | null>(null);
  const [state, dispatch] = useReducer(configureFormReducer, {
    values: toValues(args.config),
    errors: {},
    capabilities: null,
  });

  const allAgentNames = useMemo(() => args.agents.map((agent) => agent.name), [args.agents]);
  const agentsNotInChain = useMemo(
    () => allAgentNames.filter((agentName) => !state.values.fallbackChain.includes(agentName)),
    [allAgentNames, state.values.fallbackChain],
  );

  useEffect(() => {
    if (args.configLoaded) return;
    let cancelled = false;
    getDefaultConfig()
      .then((response) => {
        if (cancelled) return;
        const defaults = response.config as WizardConfig;
        args.setFullConfig(defaults);
        dispatch({ type: "set_values", values: toValues(defaults) });
      })
      .catch((caughtError: unknown) => {
        reportError("useConfigureForm.getDefaultConfig", caughtError);
      });
    return () => {
      cancelled = true;
    };
  }, [args.configLoaded, args.setFullConfig]);

  useEffect(() => {
    let cancelled = false;
    args.setDetecting(true);
    detectAgents()
      .then((detected) => {
        if (cancelled) return;
        args.setAgents(
          detected.map((agent) => ({
            name: agent.name,
            installed: agent.available,
            version: agent.version,
          })),
        );
      })
      .catch((caughtError: unknown) => {
        reportError("useConfigureForm.detectAgents", caughtError);
        if (cancelled) return;
        args.setAgents([]);
      })
      .finally(() => {
        if (!cancelled) args.setDetecting(false);
      });
    return () => {
      cancelled = true;
    };
  }, [args.setAgents, args.setDetecting]);

  useEffect(() => {
    let cancelled = false;
    resolveAgentSelection(
      state.values.executeAgent,
      state.values.executeModel,
      state.values.executeEffort,
    )
      .then((result) => {
        if (cancelled) return;
        dispatch({ type: "set_capabilities", capabilities: result.capabilities });
        dispatch({ type: "set_value", field: "executeModel", value: result.resolvedModel });
        dispatch({ type: "set_value", field: "executeEffort", value: result.resolvedEffort });
      })
      .catch((caughtError: unknown) => {
        reportError("useConfigureForm.resolveAgentSelection", caughtError);
        dispatch({ type: "set_capabilities", capabilities: null });
      });
    return () => {
      cancelled = true;
    };
  }, [state.values.executeAgent, state.values.executeModel, state.values.executeEffort]);

  const handlers: ConfigureFormHandlers = {
    onValueChange: (field, value) => {
      dispatch({ type: "set_value", field, value });
    },
    onAddAgentToChain: () => {
      if (!state.values.newAgent || state.values.fallbackChain.includes(state.values.newAgent))
        return;
      dispatch({
        type: "set_value",
        field: "fallbackChain",
        value: [...state.values.fallbackChain, state.values.newAgent],
      });
      dispatch({ type: "set_value", field: "newAgent", value: "" });
    },
    onRemoveFromChain: (agentName) => {
      dispatch({
        type: "set_value",
        field: "fallbackChain",
        value: state.values.fallbackChain.filter(
          (existingAgentName) => existingAgentName !== agentName,
        ),
      });
    },
    onDragStart: (index) => {
      dragIndexRef.current = index;
    },
    onDragOver: (event: DragEvent<HTMLDivElement>) => {
      event.preventDefault();
    },
    onDrop: (toIndex) => {
      if (dragIndexRef.current === null || dragIndexRef.current === toIndex) return;
      const reorderedChain = [...state.values.fallbackChain];
      const [movedAgent] = reorderedChain.splice(dragIndexRef.current, 1);
      reorderedChain.splice(toIndex, 0, movedAgent);
      dragIndexRef.current = null;
      dispatch({ type: "set_value", field: "fallbackChain", value: reorderedChain });
    },
    onNext: () => {
      if (!args.projectId) return;
      void completeConfigureStep(args.projectId, {
        executeAgent: state.values.executeAgent,
        executeModel: state.values.executeModel,
        executeEffort: state.values.executeEffort,
        fallbackChain: state.values.fallbackChain,
        gutterThreshold: state.values.gutterThreshold,
        maxIterations: state.values.maxIterations,
        cooldownSeconds: state.values.cooldownSeconds,
        testCommand: state.values.testCommand,
        maxVerificationRetries: state.values.maxVerificationRetries,
        scmProvider: state.values.scmProvider,
        reviewPollingInterval: state.values.reviewPollingInterval,
        reviewTimeout: state.values.reviewTimeout,
      })
        .then((result) => {
          if (Object.keys(result.errors).length > 0) {
            dispatch({ type: "set_errors", errors: result.errors as ConfigureErrors });
            return;
          }
          dispatch({ type: "set_errors", errors: {} });
          const saved = result.config;
          args.setConfig({
            executeAgent: saved.executeAgent,
            executeModel: saved.executeModel ?? null,
            executeEffort: saved.executeEffort ?? null,
            fallbackChain: saved.fallbackChain,
            gutterThreshold: saved.gutterThreshold,
            maxIterations: saved.maxIterations,
            cooldownSeconds: saved.cooldownSeconds,
            testCommand: saved.testCommand,
            maxVerificationRetries: saved.maxVerificationRetries,
            scmProvider: saved.scmProvider as ScmProvider,
            reviewPollingInterval: saved.reviewPollingInterval,
            reviewTimeout: saved.reviewTimeout,
          });
          navigate(result.nextRoute);
        })
        .catch((caughtError: unknown) => {
          reportError("useConfigureForm.completeConfigureStep", caughtError);
        });
    },
  };

  const options: ConfigureFormOptions = {
    capabilities: state.capabilities,
    errors: state.errors,
    selectableAgentNames: allAgentNames,
    agentsNotInChain,
  };

  return { values: state.values, options, handlers };
}
