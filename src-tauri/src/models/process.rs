use serde::Serialize;

use super::{LogEntry, Project};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DashboardData {
    pub(crate) running_projects: usize,
    pub(crate) stopped_projects: usize,
    pub(crate) used_ports: Vec<u16>,
    pub(crate) node_version: String,
    pub(crate) npm_version: String,
    pub(crate) pnpm_version: String,
    pub(crate) git_version: String,
    pub(crate) php_version: String,
    pub(crate) runtime_status: String,
    pub(crate) recent_errors: Vec<LogEntry>,
    pub(crate) recent_projects: Vec<Project>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ServerProcess {
    pub(crate) project_id: String,
    pub(crate) project_name: String,
    pub(crate) project_type: String,
    pub(crate) pid: u32,
    pub(crate) port: u16,
    pub(crate) url: String,
    pub(crate) network_url: String,
    pub(crate) status: String,
    pub(crate) command: String,
    pub(crate) cwd: String,
    pub(crate) started_at: String,
    pub(crate) memory_usage_mb: Option<f32>,
    pub(crate) cpu_usage: Option<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PortInfo {
    pub(crate) port: u16,
    pub(crate) available: bool,
    pub(crate) pid: Option<u32>,
    pub(crate) project_id: Option<String>,
    pub(crate) project_name: Option<String>,
    pub(crate) external: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DiagnosticItem {
    pub(crate) name: String,
    pub(crate) status: String,
    pub(crate) version: String,
    pub(crate) path: String,
    pub(crate) error: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RuntimeInfo {
    pub(crate) name: String,
    pub(crate) found: bool,
    pub(crate) version: Option<String>,
    pub(crate) path: Option<String>,
    pub(crate) source: String,
    pub(crate) last_checked_at: String,
    pub(crate) error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProxyStatus {
    pub(crate) project_id: String,
    pub(crate) running: bool,
    pub(crate) proxy_port: Option<u16>,
    pub(crate) target_port: Option<u16>,
    pub(crate) preview_url: Option<String>,
    pub(crate) error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TerminalRunResult {
    pub(crate) project_id: String,
    pub(crate) task: String,
    pub(crate) command: String,
    pub(crate) cwd: String,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) exit_code: Option<i32>,
    pub(crate) timed_out: bool,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
}
