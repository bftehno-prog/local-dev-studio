use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Project {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) project_type: String,
    pub(crate) port: Option<u16>,
    pub(crate) command: Option<String>,
    pub(crate) status: String,
    pub(crate) package_manager: Option<String>,
    pub(crate) use_docker: bool,
    pub(crate) dev_port: Option<u16>,
    pub(crate) proxy_port: Option<u16>,
    pub(crate) last_started_at: Option<String>,
    pub(crate) last_error: Option<String>,
    pub(crate) use_turbopack: bool,
    pub(crate) trusted: bool,
    pub(crate) trusted_at: Option<String>,
    pub(crate) trusted_runtime: Option<String>,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CreateProjectRequest {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) project_type: String,
    pub(crate) package_manager: Option<String>,
    #[serde(default)]
    pub(crate) auto_install: bool,
    #[serde(default)]
    pub(crate) auto_start: bool,
    #[serde(default)]
    pub(crate) use_docker: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct UpdateProjectRequest {
    pub(crate) id: String,
    pub(crate) name: Option<String>,
    pub(crate) package_manager: Option<String>,
    pub(crate) use_docker: Option<bool>,
    pub(crate) dev_port: Option<u16>,
    pub(crate) proxy_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TemplateInfo {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) project_type: String,
    pub(crate) built_in: bool,
    pub(crate) path: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TemplateManifest {
    pub(crate) name: String,
    #[serde(rename = "type")]
    pub(crate) project_type: String,
    pub(crate) version: Option<String>,
    pub(crate) author: Option<String>,
    pub(crate) description: Option<String>,
    #[serde(rename = "defaultPort")]
    pub(crate) default_port: Option<u16>,
    #[serde(rename = "packageManager")]
    pub(crate) package_manager: Option<String>,
    #[serde(rename = "requiresInstall")]
    pub(crate) requires_install: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProjectDoctorCheck {
    pub(crate) label: String,
    pub(crate) status: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProjectDoctorReport {
    pub(crate) project_id: String,
    pub(crate) project_name: String,
    pub(crate) checks: Vec<ProjectDoctorCheck>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HostingCompatibilityReport {
    pub(crate) project_id: String,
    pub(crate) project_name: String,
    pub(crate) checks: Vec<ProjectDoctorCheck>,
}
