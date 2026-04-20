use rusqlite::Connection;
use std::path::{Path, PathBuf};

const LEGACY_APP_IDS: [&str; 2] = ["com.loopforge.app", "com.tauri.dev"];

pub fn migrate_legacy_storage_if_needed(
    current_data_dir: &Path,
    current_db_path: &Path,
) -> Result<(), String> {
    let current_count = project_count(current_db_path);
    if current_count > 0 {
        return Ok(());
    }

    let Some(source_dir) = best_legacy_dir(current_data_dir) else {
        return Ok(());
    };

    let source_db = source_dir.join("loopforge.db");
    copy_db_bundle(&source_db, current_db_path)?;

    let source_projects = source_dir.join("projects");
    let current_projects = current_data_dir.join("projects");
    copy_projects_dir(&source_projects, &current_projects)?;

    Ok(())
}

fn best_legacy_dir(current_data_dir: &Path) -> Option<PathBuf> {
    let parent = current_data_dir.parent()?;
    let mut best_count = 0;
    let mut best_dir: Option<PathBuf> = None;

    for app_id in LEGACY_APP_IDS {
        let candidate_dir = parent.join(app_id);
        if candidate_dir == current_data_dir {
            continue;
        }
        let candidate_db = candidate_dir.join("loopforge.db");
        let count = project_count(&candidate_db);
        if count > best_count {
            best_count = count;
            best_dir = Some(candidate_dir);
        }
    }

    best_dir
}

fn project_count(db_path: &Path) -> i64 {
    if !db_path.exists() {
        return 0;
    }

    let Ok(conn) = Connection::open(db_path) else {
        return 0;
    };

    conn.query_row("SELECT COUNT(*) FROM projects", [], |row| {
        row.get::<_, i64>(0)
    })
    .unwrap_or(0)
}

fn copy_db_bundle(source_db: &Path, target_db: &Path) -> Result<(), String> {
    if !source_db.exists() {
        return Err(format!(
            "Legacy database not found: {}",
            source_db.display()
        ));
    }

    if let Some(parent) = target_db.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("Failed to create target data dir: {err}"))?;
    }

    remove_file_if_exists(target_db)?;
    remove_file_if_exists(&sidecar_path(target_db, "-wal"))?;
    remove_file_if_exists(&sidecar_path(target_db, "-shm"))?;

    std::fs::copy(source_db, target_db)
        .map_err(|err| format!("Failed to copy legacy database: {err}"))?;

    for suffix in ["-wal", "-shm"] {
        let source_sidecar = sidecar_path(source_db, suffix);
        if source_sidecar.exists() {
            let target_sidecar = sidecar_path(target_db, suffix);
            std::fs::copy(&source_sidecar, &target_sidecar)
                .map_err(|err| format!("Failed to copy legacy database sidecar: {err}"))?;
        }
    }

    Ok(())
}

fn sidecar_path(db_path: &Path, suffix: &str) -> PathBuf {
    let file_name = db_path.file_name().map_or_else(
        || "loopforge.db".to_string(),
        |name| name.to_string_lossy().to_string(),
    );
    db_path.with_file_name(format!("{file_name}{suffix}"))
}

fn remove_file_if_exists(path: &Path) -> Result<(), String> {
    if path.exists() {
        std::fs::remove_file(path)
            .map_err(|err| format!("Failed removing existing file {}: {err}", path.display()))?;
    }
    Ok(())
}

fn copy_projects_dir(source: &Path, target: &Path) -> Result<(), String> {
    if !source.exists() {
        return Ok(());
    }

    std::fs::create_dir_all(target)
        .map_err(|err| format!("Failed to create target projects dir: {err}"))?;

    copy_dir_recursive(source, target)
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<(), String> {
    for entry_result in
        std::fs::read_dir(source).map_err(|err| format!("Failed reading source dir: {err}"))?
    {
        let entry = entry_result.map_err(|err| format!("Failed reading source entry: {err}"))?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let file_type = entry
            .file_type()
            .map_err(|err| format!("Failed reading source entry type: {err}"))?;

        if file_type.is_dir() {
            std::fs::create_dir_all(&target_path)
                .map_err(|err| format!("Failed creating target directory: {err}"))?;
            copy_dir_recursive(&source_path, &target_path)?;
            continue;
        }

        if file_type.is_file() && !target_path.exists() {
            std::fs::copy(&source_path, &target_path)
                .map_err(|err| format!("Failed copying project artifact file: {err}"))?;
        }
    }

    Ok(())
}
