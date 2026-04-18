use super::backend_adapter::BackendAdapter;
use loopforge_app_core::atomizer::{
    AtomizerEvent, AtomizerProgress, AtomizerRequest, AtomizerRunResult, AtomizerStage,
};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomizationArtifacts {
    pub prd_path: PathBuf,
    pub prompt_path: PathBuf,
    pub guardrails_path: PathBuf,
}

pub type AtomizationRun = AtomizerRunResult<AtomizationArtifacts>;

#[derive(Debug, Clone)]
pub struct AtomizationService {
    projects_root: PathBuf,
}

impl AtomizationService {
    pub fn new(projects_root: PathBuf) -> Self {
        Self { projects_root }
    }

    pub fn stages(&self, project_id: &str) -> io::Result<Vec<AtomizerStage>> {
        BackendAdapter::atomizer_stages(
            AtomizerRequest {
                project_id: project_id.to_owned(),
            },
            |_| Ok::<_, io::Error>(AtomizerStage::ordered().into_iter().collect()),
        )
    }

    pub fn run_pipeline(&self, project_id: &str) -> io::Result<AtomizationRun> {
        let request = AtomizerRequest {
            project_id: project_id.to_owned(),
        };
        let stages = self.stages(project_id)?;
        BackendAdapter::run_atomizer(request, |request| {
            let project_dir = self.projects_root.join(&request.project_id);
            fs::create_dir_all(&project_dir)?;
            let plan_content = Self::read_plan(project_dir.join("plan.md"))?;
            let artifacts =
                Self::write_artifacts(&project_dir, &request.project_id, &plan_content)?;
            Ok(AtomizerRunResult {
                output: artifacts,
                events: Self::build_events(&request.project_id, stages),
            })
        })
    }

    pub fn progress(run: &AtomizationRun) -> Vec<AtomizerProgress> {
        BackendAdapter::atomizer_progress(&run.events)
    }

    fn write_artifacts(
        project_dir: &Path,
        project_id: &str,
        plan_content: &str,
    ) -> io::Result<AtomizationArtifacts> {
        let prd_path = project_dir.join("prd.json");
        let prompt_path = project_dir.join("prompt.md");
        let guardrails_path = project_dir.join("guardrails.md");
        fs::write(&prd_path, Self::render_prd(project_id, plan_content))?;
        fs::write(&prompt_path, Self::render_prompt(plan_content))?;
        fs::write(&guardrails_path, Self::render_guardrails(plan_content))?;
        Ok(AtomizationArtifacts {
            prd_path,
            prompt_path,
            guardrails_path,
        })
    }

    fn build_events(project_id: &str, stages: Vec<AtomizerStage>) -> Vec<AtomizerEvent> {
        let completed_stage = stages
            .last()
            .cloned()
            .unwrap_or(AtomizerStage::WriteStories);
        let mut events = stages
            .into_iter()
            .map(|stage| AtomizerEvent::StageStarted {
                project_id: project_id.to_owned(),
                stage,
            })
            .collect::<Vec<_>>();
        events.push(AtomizerEvent::StageCompleted {
            project_id: project_id.to_owned(),
            stage: completed_stage,
            story_count: Some(1),
        });
        events
    }

    fn read_plan(plan_path: PathBuf) -> io::Result<String> {
        if plan_path.exists() {
            fs::read_to_string(plan_path)
        } else {
            Ok(String::from("No plan content was available."))
        }
    }

    fn render_prd(project_id: &str, plan_content: &str) -> String {
        let objective = Self::json_escape(&Self::extract_objective(plan_content));
        format!(
            "{{\n  \"projectId\": \"{}\",\n  \"stories\": [\n    {{\n      \"id\": \"S-001\",\n      \"title\": \"Implement plan objective\",\n      \"description\": \"{}\"\n    }}\n  ]\n}}\n",
            Self::json_escape(project_id),
            objective
        )
    }

    fn render_prompt(plan_content: &str) -> String {
        format!(
            "# Execution Prompt\n\nImplement the plan objective:\n{}\n",
            Self::extract_objective(plan_content)
        )
    }

    fn render_guardrails(plan_content: &str) -> String {
        format!(
            "# Guardrails\n\n- Keep implementation aligned with objective.\n- Validate each stage output.\n- Objective: {}\n",
            Self::extract_objective(plan_content)
        )
    }

    fn extract_objective(plan_content: &str) -> String {
        plan_content
            .lines()
            .find_map(|line| {
                line.strip_prefix("- Objective: ")
                    .map(|value| value.trim().to_owned())
            })
            .unwrap_or_else(|| String::from("Ship the planned implementation safely."))
    }

    fn json_escape(value: &str) -> String {
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
    }
}

#[cfg(test)]
mod tests {
    use super::AtomizationService;
    use loopforge_app_core::atomizer::AtomizerStage;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_projects_root() -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "loopforge-shell-atomization-{}-{}",
            std::process::id(),
            stamp
        ))
    }

    #[test]
    fn run_pipeline_uses_shared_atomizer_contracts() {
        let projects_root = temp_projects_root();
        let project_id = "project-native";
        let project_dir = projects_root.join(project_id);
        fs::create_dir_all(&project_dir).expect("mkdir");
        fs::write(
            project_dir.join("plan.md"),
            "- Objective: Keep users in native shell.\n",
        )
        .expect("plan");
        let service = AtomizationService::new(projects_root.clone());
        let run = service.run_pipeline(project_id).expect("run");
        let stages = service.stages(project_id).expect("stages");
        let progress = AtomizationService::progress(&run);

        assert_eq!(stages, AtomizerStage::ordered().to_vec());
        assert_eq!(progress.len(), 5);
        assert_eq!(progress[4].message, "Done — 1 stories");
        assert_eq!(progress[0].stage_name, "summarize");
        assert!(run.output.prd_path.exists());
        assert!(run.output.prompt_path.exists());
        assert!(run.output.guardrails_path.exists());
        let prd_content = fs::read_to_string(run.output.prd_path).expect("prd");
        assert!(prd_content.contains("Keep users in native shell."));
        let _ = fs::remove_dir_all(projects_root);
    }
}
