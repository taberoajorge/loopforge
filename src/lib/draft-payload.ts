import { useWizardStore } from "../stores/wizardStore";

interface DraftPayload {
  version: number;
  projectId: string;
  currentStep: string;
  describe: {
    name: string;
    description: string;
    workingDirectory: string;
    planAgent: string;
    planModel: string | null;
    planEffort: string | null;
  };
  plan: { completed: boolean };
  atomize: { storiesCount: number };
  configure: ReturnType<typeof useWizardStore.getState>["config"];
}

export function buildDraftPayload(projectId: string, currentStep: string): string {
  const state = useWizardStore.getState();
  const draft: DraftPayload = {
    version: 1,
    projectId,
    currentStep,
    describe: {
      name: state.projectData.name,
      description: state.projectData.description,
      workingDirectory: state.projectData.workingDirectory,
      planAgent: state.projectData.planAgent,
      planModel: state.projectData.planModel,
      planEffort: state.projectData.planEffort,
    },
    plan: { completed: state.planComplete },
    atomize: { storiesCount: state.stories.length },
    configure: state.config,
  };
  return JSON.stringify(draft, null, 2);
}
