use crate::db::DbState;
use crate::projects::artifacts::{artifact_dir, non_empty_file_content};
use crate::projects::repository::{row_to_project, PROJECT_COLUMNS};
use crate::projects::wizard_state::CanonicalWizardSession;
use crate::projects::{ProjectError, WizardHydrationResult, WizardProjectData, WizardResumeState};
use ralph_core::prd::Prd;
use std::path::Path;
use tauri::{AppHandle, Manager, Runtime, State};

pub async fn finalize_draft<R: Runtime>(
    app: AppHandle<R>,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<(), ProjectError> {
    let now = chrono::Utc::now().to_rfc3339();
    let conn =
        db.0.lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
    conn.execute(
        "UPDATE projects SET wizard_step = NULL, wizard_state_json = NULL, updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, project_id],
    )?;
    drop(conn);
    let artifacts = artifact_dir(&app, &project_id)?;
    let draft_path = artifacts.join("draft.json");
    if draft_path.exists() {
        let _ = std::fs::remove_file(draft_path);
    }
    Ok(())
}

pub async fn discard_draft<R: Runtime>(
    app: AppHandle<R>,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<(), ProjectError> {
    let conn =
        db.0.lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
    let deleted = conn.execute(
        "DELETE FROM projects WHERE id = ?1 AND status = 'draft'",
        rusqlite::params![project_id],
    )?;
    drop(conn);
    if deleted == 0 {
        return Err(ProjectError::NotFound(project_id));
    }
    let dir = artifact_dir(&app, &project_id)?;
    if dir.exists() {
        let _ = std::fs::remove_dir_all(&dir);
    }
    Ok(())
}

pub async fn save_wizard_state(
    db: State<'_, DbState>,
    project_id: String,
    wizard_step: String,
    wizard_state_json: String,
) -> Result<(), ProjectError> {
    let canonical_json =
        canonical_wizard_json(&wizard_state_json, &project_id, Some(&wizard_step))?;
    let now = chrono::Utc::now().to_rfc3339();
    let conn =
        db.0.lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
    let updated = conn.execute(
        "UPDATE projects SET wizard_step = ?1, wizard_state_json = ?2, updated_at = ?3 WHERE id = ?4",
        rusqlite::params![wizard_step, canonical_json, now, project_id],
    )?;
    if updated == 0 {
        return Err(ProjectError::NotFound(project_id));
    }
    Ok(())
}

pub async fn save_draft<R: Runtime>(
    app: AppHandle<R>,
    project_id: String,
    draft_json: String,
) -> Result<(), ProjectError> {
    let artifacts = artifact_dir(&app, &project_id)?;
    std::fs::create_dir_all(&artifacts)?;
    let session = canonical_wizard_session(&draft_json, &project_id, None)?;
    std::fs::write(artifacts.join("draft.json"), session.to_json()?)?;
    let now = chrono::Utc::now().to_rfc3339();
    let db = app.state::<DbState>();
    let conn =
        db.0.lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
    let _ = conn.execute(
        "UPDATE projects SET wizard_step = ?1, wizard_state_json = NULL, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![session.current_step, now, project_id],
    );
    Ok(())
}

fn canonical_wizard_session(
    payload_json: &str,
    project_id: &str,
    fallback_step: Option<&str>,
) -> Result<CanonicalWizardSession, ProjectError> {
    Ok(CanonicalWizardSession::from_payload(
        payload_json,
        Some(project_id),
        fallback_step,
    )?)
}

fn canonical_wizard_json(
    payload_json: &str,
    project_id: &str,
    fallback_step: Option<&str>,
) -> Result<String, ProjectError> {
    Ok(canonical_wizard_session(payload_json, project_id, fallback_step)?.to_json()?)
}

pub async fn load_draft<R: Runtime>(
    app: AppHandle<R>,
    project_id: String,
) -> Result<Option<String>, ProjectError> {
    let artifacts = artifact_dir(&app, &project_id)?;
    non_empty_file_content(&artifacts.join("draft.json"))
}

pub async fn resume_wizard<R: Runtime>(
    app: AppHandle<R>,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<WizardResumeState, ProjectError> {
    let (project, wizard_state_json) = {
        let conn =
            db.0.lock()
                .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
        let query =
            format!("SELECT {PROJECT_COLUMNS}, wizard_state_json FROM projects WHERE id = ?1");
        let mut stmt = conn.prepare(&query)?;
        stmt.query_row(rusqlite::params![project_id], |row| {
            Ok((row_to_project(row)?, row.get::<_, Option<String>>(8)?))
        })
        .map_err(|_| ProjectError::NotFound(project_id.clone()))?
    };

    let fallback_step = project
        .wizard_step
        .clone()
        .or_else(|| (project.status == "draft").then(|| "describe".to_string()));
    let dir = artifact_dir(&app, &project_id)?;
    let draft_json = non_empty_file_content(&dir.join("draft.json"))?;
    let session_source = draft_json.clone().or_else(|| wizard_state_json.clone());
    let wizard_session = session_source
        .as_deref()
        .map(|raw| canonical_wizard_session(raw, &project_id, fallback_step.as_deref()))
        .transpose()?;
    let wizard_step = wizard_session
        .as_ref()
        .map(|session| session.current_step.clone())
        .or(fallback_step)
        .ok_or_else(|| ProjectError::NotFound(format!("No wizard state for {project_id}")))?;

    let artifact_plan = non_empty_file_content(&dir.join("plan.md"))?;
    let legacy_plan =
        non_empty_file_content(&Path::new(&project.working_directory).join("plan.md"))?;
    let has_plan = artifact_plan.is_some()
        || legacy_plan.is_some()
        || wizard_session
            .as_ref()
            .and_then(|session| session.plan.document.as_ref())
            .is_some()
        || wizard_session
            .as_ref()
            .is_some_and(|session| session.plan.completed);

    let artifact_prd = Prd::load(&dir.join("prd.json"))
        .ok()
        .filter(|prd| !prd.stories.is_empty());
    let legacy_prd = Prd::load(&Path::new(&project.working_directory).join("prd.json"))
        .ok()
        .filter(|prd| !prd.stories.is_empty());
    let has_prd = artifact_prd.is_some()
        || legacy_prd.is_some()
        || wizard_session
            .as_ref()
            .is_some_and(|session| !session.atomize.stories.is_empty());

    Ok(WizardResumeState {
        project,
        wizard_step,
        wizard_session: wizard_session.clone(),
        wizard_state_json: wizard_session
            .map(|session| session.to_json())
            .transpose()?,
        has_plan,
        has_prd,
    })
}

pub async fn hydrate_wizard<R: Runtime>(
    app: AppHandle<R>,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<WizardHydrationResult, ProjectError> {
    let resume = resume_wizard(app.clone(), db, project_id.clone()).await?;
    let dir = artifact_dir(&app, &project_id)?;
    let session = resume.wizard_session.clone().unwrap_or_default();
    let current_step_number =
        crate::projects::wizard_state::step_name_to_number(&resume.wizard_step).unwrap_or(1);
    let highest_step = session.highest_step.max(current_step_number);

    let project_data = hydrated_project_data(&resume.project, &session);
    let plan_complete = resume.has_plan || session.plan.completed;
    let stories = load_hydrated_stories(&dir, &session, resume.has_prd);
    let config = session.configure.clone();

    Ok(WizardHydrationResult {
        project: resume.project,
        wizard_step: resume.wizard_step,
        highest_step,
        project_data,
        plan_complete,
        stories,
        config,
    })
}

fn hydrated_project_data(
    project: &crate::projects::Project,
    session: &CanonicalWizardSession,
) -> WizardProjectData {
    WizardProjectData {
        name: non_empty_or(&session.describe.name, &project.name),
        description: non_empty_or(&session.describe.description, &project.description),
        working_directory: non_empty_or(
            &session.describe.working_directory,
            &project.working_directory,
        ),
        plan_agent: non_empty_or(&session.describe.plan_agent, "claude"),
        plan_model: session.describe.plan_model.clone(),
        plan_effort: session.describe.plan_effort.clone(),
    }
}

fn non_empty_or(value: &str, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}

fn load_hydrated_stories(
    dir: &Path,
    session: &CanonicalWizardSession,
    has_prd: bool,
) -> Vec<ralph_core::prd::UserStory> {
    if has_prd {
        return Prd::load(&dir.join("prd.json"))
            .map(|prd| prd.stories)
            .unwrap_or_default();
    }
    session.atomize.stories.clone()
}
