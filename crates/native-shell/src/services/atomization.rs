use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomizationStage {
    StageOne,
    StageTwo,
    StageThree,
    StageFour,
}

impl AtomizationStage {
    pub fn label(self) -> String {
        match self {
            Self::StageOne => String::from("Stage 1"),
            Self::StageTwo => String::from("Stage 2"),
            Self::StageThree => String::from("Stage 3"),
            Self::StageFour => String::from("Stage 4"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomizationProgress {
    pub stage: AtomizationStage,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomizationArtifacts {
    pub prd_path: PathBuf,
    pub prompt_path: PathBuf,
    pub guardrails_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomizationRun {
    pub progress: Vec<AtomizationProgress>,
    pub artifacts: AtomizationArtifacts,
}

#[derive(Debug, Clone)]
pub struct AtomizationService {
    projects_root: PathBuf,
}

impl AtomizationService {
    pub fn new(projects_root: PathBuf) -> Self {
        Self { projects_root }
    }

    pub fn run_pipeline(&self, project_id: &str) -> io::Result<AtomizationRun> {
        let project_dir = self.projects_root.join(project_id);
        fs::create_dir_all(&project_dir)?;
        let plan_content = Self::read_plan(project_dir.join("plan.md"))?;
        let progress = vec![
            AtomizationProgress {
                stage: AtomizationStage::StageOne,
                detail: String::from("Loaded plan input from plan.md."),
            },
            AtomizationProgress {
                stage: AtomizationStage::StageTwo,
                detail: String::from("Generated structured stories for prd.json."),
            },
            AtomizationProgress {
                stage: AtomizationStage::StageThree,
                detail: String::from("Built execution prompt artifact."),
            },
            AtomizationProgress {
                stage: AtomizationStage::StageFour,
                detail: String::from("Materialized guardrails artifact."),
            },
        ];
        let prd_path = project_dir.join("prd.json");
        let prompt_path = project_dir.join("prompt.md");
        let guardrails_path = project_dir.join("guardrails.md");
        fs::write(&prd_path, Self::render_prd(project_id, &plan_content))?;
        fs::write(&prompt_path, Self::render_prompt(&plan_content))?;
        fs::write(&guardrails_path, Self::render_guardrails(&plan_content))?;
        Ok(AtomizationRun {
            progress,
            artifacts: AtomizationArtifacts {
                prd_path,
                prompt_path,
                guardrails_path,
            },
        })
    }

    fn read_plan(plan_path: PathBuf) -> io::Result<String> {
        if plan_path.exists() {
            fs::read_to_string(plan_path)
        } else {
            Ok(String::from("No plan content was available."))
        }
    }

    fn render_prd(project_id: &str, plan_content: &str) -> String {
        let objective = Self::extract_objective(plan_content);
        let objective = Self::json_escape(&objective);
        format!(
            "{{\n  \"projectId\": \"{}\",\n  \"stories\": [\n    {{\n      \"id\": \"S-001\",\n      \"title\": \"Implement plan objective\",\n      \"description\": \"{}\"\n    }}\n  ]\n}}\n",
            Self::json_escape(project_id),
            objective
        )
    }

    fn render_prompt(plan_content: &str) -> String {
        let objective = Self::extract_objective(plan_content);
        format!(
            "# Execution Prompt\n\nImplement the plan objective:\n{}\n",
            objective
        )
    }

    fn render_guardrails(plan_content: &str) -> String {
        let objective = Self::extract_objective(plan_content);
        format!(
            "# Guardrails\n\n- Keep implementation aligned with objective.\n- Validate each stage output.\n- Objective: {}\n",
            objective
        )
    }

    fn extract_objective(plan_content: &str) -> String {
        for line in plan_content.lines() {
            if let Some(value) = line.strip_prefix("- Objective: ") {
                return value.trim().to_owned();
            }
        }
        String::from("Ship the planned implementation safely.")
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
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};
    use super::{AtomizationService, AtomizationStage};

    fn temp_projects_root() -> std::path::PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("time").as_nanos();
        std::env::temp_dir().join(format!("loopforge-shell-atomization-{}-{}", std::process::id(), stamp))
    }

    #[test]
    fn run_pipeline_persists_artifacts_and_returns_stage_progress() {
        let projects_root = temp_projects_root();
        let project_id = "project-native";
        let project_dir = projects_root.join(project_id);
        fs::create_dir_all(&project_dir).expect("mkdir");
        fs::write(project_dir.join("plan.md"), "- Objective: Keep users in native shell.\n").expect("plan");
        let service = AtomizationService::new(projects_root.clone());
        let run = service.run_pipeline(project_id).expect("run");
        assert_eq!(run.progress.len(), 4);
        assert_eq!(run.progress[0].stage, AtomizationStage::StageOne);
        assert_eq!(run.progress[3].stage, AtomizationStage::StageFour);
        assert!(run.artifacts.prd_path.exists());
        assert!(run.artifacts.prompt_path.exists());
        assert!(run.artifacts.guardrails_path.exists());
        let prd_content = fs::read_to_string(run.artifacts.prd_path).expect("prd");
        assert!(prd_content.contains("Keep users in native shell."));
        let _ = fs::remove_dir_all(projects_root);
    }
}
