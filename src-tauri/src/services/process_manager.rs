use std::{
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use rusqlite::{params, Connection, OptionalExtension};
use sysinfo::{Pid, System};

use crate::{
    models::ServerProcess,
    services::port_manager::network_url,
    utils::{network::is_port_free, time::now},
};

pub(crate) fn list_servers(db_path: &Path) -> Result<Vec<ServerProcess>, String> {
    let conn = connect(db_path)?;
    let mut sys = System::new_all();
    sys.refresh_all();
    let mut stmt = conn
        .prepare(
            "SELECT p.project_id, pr.name, pr.project_type, p.pid, p.port, p.command, p.cwd, p.started_at, p.status
             FROM processes p JOIN projects pr ON pr.id = p.project_id ORDER BY p.started_at DESC",
        )
        .map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            let pid_u32: u32 = row.get(3)?;
            let process = sys.process(Pid::from_u32(pid_u32));
            let port = row.get::<_, u16>(4)?;
            Ok((
                process.is_some(),
                ServerProcess {
                    project_id: row.get(0)?,
                    project_name: row.get(1)?,
                    project_type: row.get(2)?,
                    pid: pid_u32,
                    port,
                    url: format!("http://localhost:{}", port),
                    network_url: network_url(port),
                    status: row.get(8)?,
                    command: row.get(5)?,
                    cwd: row.get(6)?,
                    started_at: row.get(7)?,
                    memory_usage_mb: process.map(|p| p.memory() as f32 / 1024.0 / 1024.0),
                    cpu_usage: process.map(|p| p.cpu_usage()),
                },
            ))
        })
        .map_err(|error| error.to_string())?;
    let mut servers = Vec::new();
    for row in rows {
        let (alive, server) = row.map_err(|error| error.to_string())?;
        if alive {
            servers.push(server);
        } else {
            let _ = mark_project_stopped(db_path, &server.project_id);
            let _ = insert_process_log(
                db_path,
                Some(&server.project_id),
                "warning",
                "Removed stale process record",
            );
        }
    }
    Ok(servers)
}

pub(crate) fn monitor_project_startup(
    db_path: std::path::PathBuf,
    project_id: String,
    pid: u32,
    port: u16,
    timeout_seconds: u32,
) {
    thread::spawn(move || {
        let timeout = Duration::from_secs(timeout_seconds.max(1) as u64);
        let started = Instant::now();
        while started.elapsed() < timeout {
            if !process_exists(pid) {
                let _ = update_process_status(&db_path, &project_id, "error");
                let _ = update_project_status(&db_path, &project_id, "error");
                let _ = insert_process_log(
                    &db_path,
                    Some(&project_id),
                    "error",
                    "Process exited before the server became ready",
                );
                return;
            }
            if !is_port_free(port) {
                let _ = update_process_status(&db_path, &project_id, "running");
                let _ = update_project_status(&db_path, &project_id, "running");
                let _ = insert_process_log(
                    &db_path,
                    Some(&project_id),
                    "server",
                    &format!("Server ready on port {}", port),
                );
                return;
            }
            thread::sleep(Duration::from_millis(350));
        }
        let _ = update_process_status(&db_path, &project_id, "error");
        let _ = update_project_status(&db_path, &project_id, "error");
        let _ = insert_process_log(
            &db_path,
            Some(&project_id),
            "error",
            &format!(
                "Server did not open port {} within {} seconds.",
                port,
                timeout_seconds.max(1)
            ),
        );
    });
}

pub(crate) fn process_exists(pid: u32) -> bool {
    let mut sys = System::new_all();
    sys.refresh_all();
    sys.process(Pid::from_u32(pid)).is_some()
}

pub(crate) fn update_process_status(
    db_path: &Path,
    project_id: &str,
    status: &str,
) -> Result<(), String> {
    connect(db_path)?
        .execute(
            "UPDATE processes SET status = ?2 WHERE project_id = ?1",
            params![project_id, status],
        )
        .map(|_| ())
        .map_err(|error| error.to_string())
}

pub(crate) fn update_project_status(
    db_path: &Path,
    project_id: &str,
    status: &str,
) -> Result<(), String> {
    connect(db_path)?
        .execute(
            "UPDATE projects SET status = ?2, updated_at = ?3 WHERE id = ?1",
            params![project_id, status, now()],
        )
        .map(|_| ())
        .map_err(|error| error.to_string())
}

pub(crate) fn stored_pid(db_path: &Path, project_id: &str) -> Result<Option<u32>, String> {
    connect(db_path)?
        .query_row(
            "SELECT pid FROM processes WHERE project_id = ?1",
            params![project_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())
}

pub(crate) fn kill_process_tree(pid: u32) {
    if cfg!(windows) {
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    } else {
        let _ = Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .status();
    }
}

pub(crate) fn mark_project_stopped(db_path: &Path, project_id: &str) -> Result<(), String> {
    connect(db_path)?
        .execute(
            "DELETE FROM processes WHERE project_id = ?1",
            params![project_id],
        )
        .map_err(|error| error.to_string())?;
    connect(db_path)?
        .execute(
            "UPDATE projects SET status = 'stopped', updated_at = ?2 WHERE id = ?1",
            params![project_id, now()],
        )
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn insert_process_log(
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

fn connect(db_path: &Path) -> Result<Connection, String> {
    Connection::open(db_path).map_err(|error| error.to_string())
}
