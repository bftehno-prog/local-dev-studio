export type ProjectType = 'next' | 'vite' | 'astro' | 'static' | 'node' | 'php' | 'unknown';
export type ProjectStatus =
  | 'idle'
  | 'starting'
  | 'running'
  | 'stopping'
  | 'stopped'
  | 'error'
  | 'installing'
  | 'building';

export interface Project {
  id: string;
  name: string;
  path: string;
  project_type: ProjectType;
  port?: number;
  command?: string;
  status: ProjectStatus;
  package_manager?: string;
  use_docker: boolean;
  dev_port?: number;
  proxy_port?: number;
  last_started_at?: string;
  last_error?: string;
  use_turbopack: boolean;
  trusted: boolean;
  trusted_at?: string;
  trusted_runtime?: string;
  created_at: string;
  updated_at: string;
}

export interface CreateProjectRequest {
  name: string;
  path: string;
  project_type: string;
  package_manager?: string;
  auto_install?: boolean;
  auto_start?: boolean;
  use_docker?: boolean;
}

export interface UpdateProjectRequest {
  id: string;
  name?: string;
  package_manager?: string;
  use_docker?: boolean;
  dev_port?: number;
  proxy_port?: number;
}

export interface ServerProcess {
  project_id: string;
  project_name: string;
  project_type: ProjectType;
  pid: number;
  port: number;
  url: string;
  network_url: string;
  status: ProjectStatus;
  command: string;
  cwd: string;
  started_at: string;
  memory_usage_mb?: number;
  cpu_usage?: number;
}

export interface ProxyStatus {
  project_id: string;
  running: boolean;
  proxy_port?: number;
  target_port?: number;
  preview_url?: string;
  error?: string;
}

export interface ProjectFileEntry {
  path: string;
  name: string;
  is_dir: boolean;
  size?: number;
}

export interface ProjectFileContent {
  path: string;
  content: string;
  size: number;
}

export interface LogEntry {
  id: number;
  project_id?: string;
  project_name?: string;
  level: 'info' | 'warning' | 'error' | 'build' | 'server';
  message: string;
  created_at: string;
}

export interface PortInfo {
  port: number;
  available: boolean;
  pid?: number;
  project_id?: string;
  project_name?: string;
  external: boolean;
}

export interface Settings {
  language: 'en' | 'ru';
  onboarding_completed: boolean;
  projects_folder: string;
  sandboxes_folder: string;
  package_manager: string;
  port_start: number;
  port_end: number;
  open_preview_automatically: boolean;
  start_minimized: boolean;
  launch_on_startup: boolean;
  use_bundled_node: boolean;
  node_path: string;
  npm_path: string;
  pnpm_path: string;
  yarn_path: string;
  bun_path: string;
  php_path: string;
  git_path: string;
  use_turbopack: boolean;
  clear_next_before_start: boolean;
  enable_network_preview: boolean;
  enable_https: boolean;
  default_next_port: number;
  default_device: string;
  desktop_width: number;
  laptop_width: number;
  tablet_width: number;
  mobile_width: number;
  custom_width: number;
  auto_reload_preview: boolean;
  open_external_browser_on_start: boolean;
  environment_variables: string;
  hosts: string;
  ssl_certificates: string;
  proxy_rules: string;
  process_timeout: number;
  log_retention: number;
}

export interface DashboardData {
  running_projects: number;
  stopped_projects: number;
  used_ports: number[];
  node_version: string;
  npm_version: string;
  pnpm_version: string;
  git_version: string;
  php_version: string;
  runtime_status: string;
  recent_errors: LogEntry[];
  recent_projects: Project[];
}

export interface TemplateInfo {
  id: string;
  name: string;
  project_type: ProjectType;
  built_in: boolean;
  path?: string;
}

export interface DiagnosticItem {
  name: string;
  status: 'OK' | 'Missing' | 'Warning' | 'Error';
  version: string;
  path: string;
  error: string;
}

export interface RuntimeInfo {
  name: string;
  found: boolean;
  version?: string;
  path?: string;
  source: 'bundled' | 'system' | 'custom';
  last_checked_at: string;
  error?: string;
}

export interface ProjectDoctorCheck {
  label: string;
  status: 'ok' | 'warning' | 'error';
  message: string;
}

export interface ProjectDoctorReport {
  project_id: string;
  project_name: string;
  checks: ProjectDoctorCheck[];
}

export interface HostingCompatibilityReport {
  project_id: string;
  project_name: string;
  checks: ProjectDoctorCheck[];
}
