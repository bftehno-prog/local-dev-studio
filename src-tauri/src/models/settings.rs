use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Settings {
    #[serde(default = "default_language")]
    pub(crate) language: String,
    #[serde(default)]
    pub(crate) onboarding_completed: bool,
    pub(crate) projects_folder: String,
    pub(crate) sandboxes_folder: String,
    pub(crate) package_manager: String,
    pub(crate) port_start: u16,
    pub(crate) port_end: u16,
    pub(crate) open_preview_automatically: bool,
    pub(crate) start_minimized: bool,
    pub(crate) launch_on_startup: bool,
    pub(crate) use_bundled_node: bool,
    pub(crate) node_path: String,
    pub(crate) npm_path: String,
    pub(crate) pnpm_path: String,
    pub(crate) yarn_path: String,
    pub(crate) bun_path: String,
    pub(crate) php_path: String,
    pub(crate) git_path: String,
    pub(crate) use_turbopack: bool,
    pub(crate) clear_next_before_start: bool,
    pub(crate) enable_network_preview: bool,
    pub(crate) enable_https: bool,
    pub(crate) default_next_port: u16,
    pub(crate) default_device: String,
    pub(crate) desktop_width: u16,
    pub(crate) laptop_width: u16,
    pub(crate) tablet_width: u16,
    pub(crate) mobile_width: u16,
    pub(crate) custom_width: u16,
    pub(crate) auto_reload_preview: bool,
    pub(crate) open_external_browser_on_start: bool,
    pub(crate) environment_variables: String,
    pub(crate) hosts: String,
    pub(crate) ssl_certificates: String,
    pub(crate) proxy_rules: String,
    pub(crate) process_timeout: u32,
    pub(crate) log_retention: u32,
}

pub(crate) fn default_language() -> String {
    "ru".to_string()
}
