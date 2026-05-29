use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

pub fn run_migrations(db_path: &Path) -> Result<(), String> {
    let mut conn = Connection::open(db_path).map_err(|error| error.to_string())?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL
        )",
        [],
    )
    .map_err(|error| format!("Failed to prepare schema migrations: {}", error))?;
    apply_migration(&mut conn, 1, "001_initial_schema", initial_schema_sql())?;
    Ok(())
}

fn apply_migration(
    conn: &mut Connection,
    version: i64,
    name: &str,
    sql: &str,
) -> Result<(), String> {
    let exists: Option<i64> = conn
        .query_row(
            "SELECT version FROM schema_migrations WHERE version = ?1",
            params![version],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    if exists.is_some() {
        return Ok(());
    }
    let tx = conn
        .transaction()
        .map_err(|error| format!("Failed to start migration {}: {}", name, error))?;
    tx.execute_batch(sql)
        .map_err(|error| format!("Failed to apply migration {}: {}", name, error))?;
    tx.execute(
        "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
        params![version, name, Utc::now().to_rfc3339()],
    )
    .map_err(|error| format!("Failed to record migration {}: {}", name, error))?;
    tx.commit()
        .map_err(|error| format!("Failed to commit migration {}: {}", name, error))?;
    Ok(())
}

fn initial_schema_sql() -> &'static str {
    "
    CREATE TABLE IF NOT EXISTS projects (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        path TEXT NOT NULL UNIQUE,
        project_type TEXT NOT NULL,
        port INTEGER,
        command TEXT,
        status TEXT NOT NULL,
        use_turbopack INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
    CREATE TABLE IF NOT EXISTS processes (
        project_id TEXT PRIMARY KEY,
        pid INTEGER NOT NULL,
        command TEXT NOT NULL,
        cwd TEXT NOT NULL,
        port INTEGER NOT NULL,
        started_at TEXT NOT NULL,
        status TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS logs (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        project_id TEXT,
        level TEXT NOT NULL,
        message TEXT NOT NULL,
        created_at TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS templates (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        project_type TEXT NOT NULL,
        built_in INTEGER NOT NULL,
        path TEXT
    );
    CREATE TABLE IF NOT EXISTS sandboxes (
        id TEXT PRIMARY KEY,
        project_id TEXT NOT NULL,
        template_id TEXT NOT NULL,
        created_at TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS ports (
        port INTEGER PRIMARY KEY,
        project_id TEXT,
        pid INTEGER,
        status TEXT NOT NULL
    );
    "
}
