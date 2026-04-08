import { useEffect, useState } from "react";
import { useWizardStore } from "../stores/wizardStore";
import { resumeWizard, loadExistingPrd, loadDraft } from "../lib/tauri";

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
    resumeWizard(urlProjectId)
      .then(async (resumeState) => {
        const store = useWizardStore.getState();
        store.setProjectId(resumeState.project.id);
        store.setProjectData({
          name: resumeState.project.name,
          description: resumeState.project.description,
          workingDirectory: resumeState.project.workingDirectory,
          planModel: null,
          planEffort: null,
        });

        const draftJson = await loadDraft(urlProjectId).catch(() => null);
        if (draftJson) {
          try {
            const draft = JSON.parse(draftJson);
            if (draft.describe) {
              store.setProjectData({
                name: draft.describe.name ?? resumeState.project.name,
                description: draft.describe.description ?? resumeState.project.description,
                workingDirectory: draft.describe.workingDirectory ?? resumeState.project.workingDirectory,
                planAgent: draft.describe.planAgent ?? store.projectData.planAgent,
                planModel: draft.describe.planModel ?? null,
                planEffort: draft.describe.planEffort ?? null,
              });
            }
            if (draft.plan?.completed) store.setPlanComplete(true);
            if (draft.configure) store.setConfig(draft.configure);
          } catch {
          }
        }

        if (resumeState.hasPlan) store.setPlanComplete(true);
        if (resumeState.hasPrd) {
          const prd = await loadExistingPrd(resumeState.project.id);
          if (prd && prd.stories.length > 0) store.setStories(prd.stories);
        }
      })
      .catch(() => {})
      .finally(() => setHydrating(false));
  }, [urlProjectId, projectId, projectName]);

  return { hydrating };
}
