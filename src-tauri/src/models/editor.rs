use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProjectFileEntry {
    pub(crate) path: String,
    pub(crate) name: String,
    pub(crate) is_dir: bool,
    pub(crate) size: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProjectFileContent {
    pub(crate) path: String,
    pub(crate) content: String,
    pub(crate) size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RecentProjectFile {
    pub(crate) project_id: String,
    pub(crate) path: String,
    pub(crate) language: Option<String>,
    pub(crate) opened_at: String,
}
