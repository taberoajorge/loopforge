import { useParams } from "react-router";
import { useAgentStore } from "../../stores/agentStore";
import { useWizardStore } from "../../stores/wizardStore";
import { ConfigureForm } from "./components/ConfigureForm";
import { useConfigureForm } from "./hooks/useConfigureForm";

export function Configure() {
  const { id } = useParams<{ id: string }>();
  const agents = useAgentStore((state) => state.agents);
  const setAgents = useAgentStore((state) => state.setAgents);
  const setDetecting = useAgentStore((state) => state.setDetecting);
  const config = useWizardStore((state) => state.config);
  const configLoaded = useWizardStore((state) => state.configLoaded);
  const setConfig = useWizardStore((state) => state.setConfig);
  const setFullConfig = useWizardStore((state) => state.setFullConfig);
  const form = useConfigureForm({
    projectId: id,
    agents,
    config,
    configLoaded,
    setConfig,
    setFullConfig,
    setAgents,
    setDetecting,
  });

  return <ConfigureForm values={form.values} options={form.options} handlers={form.handlers} />;
}
