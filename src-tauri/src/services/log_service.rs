use std::{fs, path::Path};

use chrono::Utc;
use rusqlite::{params, Connection};

use crate::{models::LogEntry, utils::time::now};

pub(crate) fn list_logs(
    db_path: &Path,
    project_id: Option<String>,
    level: Option<String>,
    search: Option<String>,
) -> Result<Vec<LogEntry>, String> {
    let conn = connect(db_path)?;
    let project_filter = project_id.filter(|value| !value.is_empty());
    let level_filter = level.filter(|value| !value.is_empty());
    let search_filter = search
        .filter(|value| !value.is_empty())
        .map(|value| format!("%{}%", value));
    let mut stmt = conn
        .prepare(
            "SELECT l.id, l.project_id, p.name, l.level, l.message, l.created_at
             FROM logs l LEFT JOIN projects p ON p.id = l.project_id
             WHERE (?1 IS NULL OR l.project_id = ?1)
               AND (?2 IS NULL OR l.level = ?2)
               AND (?3 IS NULL OR l.message LIKE ?3)
             ORDER BY l.id DESC LIMIT 500",
        )
        .map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map(
            params![project_filter, level_filter, search_filter],
            |row| {
                Ok(LogEntry {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    project_name: row.get(2)?,
                    level: row.get(3)?,
                    message: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        )
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub(crate) fn clear_logs(db_path: &Path) -> Result<(), String> {
    connect(db_path)?
        .execute("DELETE FROM logs", [])
        .map(|_| ())
        .map_err(|error| error.to_string())
}

pub(crate) fn export_logs(db_path: &Path, output_dir: &Path) -> Result<String, String> {
    let logs = list_logs(db_path, None, None, None)?;
    let path = output_dir.join(format!("logs-{}.txt", Utc::now().format("%Y%m%d-%H%M%S")));
    let body = logs
        .into_iter()
        .map(|log| {
            format!(
                "{} [{}] {} {}",
                log.created_at,
                log.level,
                log.project_name.unwrap_or_default(),
                log.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&path, body).map_err(|error| error.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

pub(crate) fn append_log(
    db_path: &Path,
    project_id: Option<&str>,
    level: &str,
    message: &str,
) -> Result<(), String> {
    connect(db_path)?
        .execute(
            "INSERT INTO logs (project_id, level, message, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![project_id, level, message, now()],
        )
        .map(|_| ())
        .map_err(|error| error.to_string())
}

pub(crate) fn prune_logs(db_path: &Path, retention_days: u32) -> Result<(), String> {
    if retention_days == 0 {
        return Ok(());
    }
    let cutoff = (Utc::now() - chrono::Duration::days(retention_days as i64)).to_rfc3339();
    connect(db_path)?
        .execute("DELETE FROM logs WHERE created_at < ?1", params![cutoff])
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn connect(db_path: &Path) -> Result<Connection, String> {
    Connection::open(db_path).map_err(|error| error.to_string())
}
