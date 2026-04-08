use crate::config::RalphConfig;
use crate::logger;
use crate::prd::Prd;

pub fn load_or_restore(config: &RalphConfig) -> Option<Prd> {
    if Prd::is_valid_json(&config.paths.prd_file) {
        return Prd::load(&config.paths.prd_file).ok();
    }
    logger::log_warning("PRD missing or corrupted, restoring from backup");
    if config.paths.prd_backup.exists() {
        if let Err(copy_err) = std::fs::copy(&config.paths.prd_backup, &config.paths.prd_file) {
            tracing::warn!(
                error = %copy_err,
                "failed to restore PRD from backup"
            );
            return None;
        }
        return Prd::load(&config.paths.prd_file).ok();
    }
    None
}

pub fn mark_story_blocked(config: &RalphConfig, story_id: &str) {
    if let Some(mut prd) = load_or_restore(config) {
        if let Some(story) = prd.stories.iter_mut().find(|st| st.id == story_id) {
            story.blocked = true;
        }
        if let Err(save_err) = prd.save(&config.paths.prd_file) {
            tracing::warn!(error = %save_err, "failed to save PRD after blocking story");
        }
    }
}

pub fn summarize_approach(output_lines: &[String]) -> String {
    let meaningful: Vec<&str> = output_lines
        .iter()
        .rev()
        .take(20)
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && trimmed.len() > 10
        })
        .take(3)
        .map(|line| line.as_str())
        .collect();

    if meaningful.is_empty() {
        return "unknown approach".to_string();
    }
    meaningful.join(" | ")
}
