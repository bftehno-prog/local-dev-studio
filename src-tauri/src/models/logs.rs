use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LogEntry {
    pub(crate) id: i64,
    pub(crate) project_id: Option<String>,
    pub(crate) project_name: Option<String>,
    pub(crate) level: String,
    pub(crate) message: String,
    pub(crate) created_at: String,
}
