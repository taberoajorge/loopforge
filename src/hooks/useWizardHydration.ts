import { useEffect, useState } from "react";
import { reportError } from "../lib/reportError";
import { hydrateWizard } from "../lib/tauri";
import { useWizardStore } from "../stores/wizardStore";

const STEP_NUMBERS: Record<string, number> = {
  describe: 1,
  plan: 2,
  atomize: 3,
  configure: 4,
  launch: 5,
};

export function useWizardHydration(
  urlProjectId: string | undefined,
  projectId: string | null,
  projectName: string,
) {
  const [hydrating, setHydrating] = useState(false);

  useEffect(() => {
    if (!urlProjectId) return;
    if (projectId === urlProjectId && projectName) return;

    setHydrating(true);
    hydrateWizard(urlProjectId)
      .then((result) => {
        const store = useWizardStore.getState();
        store.setProjectId(result.project.id);
        store.setStep(STEP_NUMBERS[result.wizardStep] ?? 1);
        store.setHighestStep(result.highestStep);
        store.setProjectData({
          name: result.projectData.name,
          description: result.projectData.description,
          workingDirectory: result.projectData.workingDirectory,
          planAgent: result.projectData.planAgent,
          planModel: result.projectData.planModel,
          planEffort: result.projectData.planEffort,
        });
        if (result.planComplete) store.setPlanComplete(true);
        if (result.stories.length > 0) store.setStories(result.stories);
        if (result.config) {
          store.setConfig(result.config as import("../types/wizard").WizardConfig);
        }
      })
      .catch((caughtError: unknown) => {
        reportError("useWizardHydration.hydrateWizard", caughtError);
      })
      .finally(() => setHydrating(false));
  }, [urlProjectId, projectId, projectName]);

  return { hydrating };
}
