use std::{
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use rusqlite::{params, Connection, OptionalExtension};
use sysinfo::{Pid, System};

use crate::utils::{network::is_port_free, time::now};

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
