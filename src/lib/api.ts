import { invoke } from '@tauri-apps/api/core';
import type {
  DashboardData,
  CreateProjectRequest,
  LogEntry,
  PortInfo,
  Project,
  ProjectDoctorReport,
  ProjectFileContent,
  ProjectFileEntry,
  HostingCompatibilityReport,
  ProxyStatus,
  ProjectType,
  ServerProcess,
  Settings,
  DiagnosticItem,
  RuntimeInfo,
  TemplateInfo,
  UpdateProjectRequest,
} from './types';

export const api = {
  dashboard: () => invoke<DashboardData>('dashboard'),
  listProjects: () => invoke<Project[]>('list_projects'),
  getProject: (id: string) => invoke<Project>('get_project', { id }),
  createProject: (request: CreateProjectRequest) => invoke<Project>('create_project', { request }),
  importProject: (path: string) => invoke<Project>('import_project', { path }),
  updateProject: (request: UpdateProjectRequest) => invoke<Project>('update_project', { request }),
  deleteProject: (id: string) => invoke<void>('delete_project', { id }),
  addProject: (path: string) => invoke<Project>('add_project', { path }),
  removeProject: (id: string) => invoke<void>('remove_project', { id }),
  startProject: (id: string) => invoke<Project>('start_project', { id }),
  installProjectDependencies: (id: string) =>
    invoke<Project>('install_project_dependencies', { id }),
  stopProject: (id: string) => invoke<Project>('stop_project', { id }),
  trustProject: (id: string) => invoke<Project>('trust_project', { id }),
  resetProjectTrust: (id: string) => invoke<Project>('reset_project_trust', { id }),
  startAllProjects: () => invoke<Project[]>('start_all_projects'),
  stopAllProjects: () => invoke<Project[]>('stop_all_projects'),
  restartProject: async (id: string) => {
    await invoke<Project>('stop_project', { id });
    return invoke<Project>('start_project', { id });
  },
  openPath: (path: string) => invoke<void>('open_path', { path }),
  openInCode: (path: string) => invoke<void>('open_in_code', { path }),
  openExternal: (url: string) => invoke<void>('open_external_url', { url }),
  networkUrl: (port: number) => invoke<string>('network_url', { port }),
  startProxy: (id: string) => invoke<ProxyStatus>('start_proxy', { id }),
  stopProxy: (id: string) => invoke<ProxyStatus>('stop_proxy', { id }),
  restartProxy: (id: string) => invoke<ProxyStatus>('restart_proxy', { id }),
  getPreviewUrl: (id: string) => invoke<string>('get_preview_url', { id }),
  getProxyStatus: (id: string) => invoke<ProxyStatus>('get_proxy_status', { id }),
  clearCache: (id: string) => invoke<void>('clear_project_cache', { id }),
  listProjectFiles: (id: string) => invoke<ProjectFileEntry[]>('list_project_files', { id }),
  readProjectFile: (id: string, path: string) =>
    invoke<ProjectFileContent>('read_project_file', { id, path }),
  writeProjectFile: (id: string, path: string, content: string) =>
    invoke<ProjectFileContent>('write_project_file', { id, path, content }),
  listServers: () => invoke<ServerProcess[]>('list_servers'),
  listPorts: () => invoke<PortInfo[]>('list_ports'),
  releasePort: (port: number) => invoke<void>('release_port', { port }),
  listLogs: (projectId?: string, level?: string, search?: string) =>
    invoke<LogEntry[]>('list_logs', { projectId, level, search }),
  clearLogs: () => invoke<void>('clear_logs'),
  exportLogs: () => invoke<string>('export_logs'),
  getSettings: () => invoke<Settings>('get_settings'),
  saveSettings: (settings: Settings) => invoke<Settings>('save_settings', { settings }),
  listTemplates: () => invoke<TemplateInfo[]>('list_templates'),
  createFromTemplate: (templateId: string, name?: string) =>
    invoke<Project>('create_from_template', { templateId, name }),
  createSandbox: (templateId: string) => invoke<Project>('create_sandbox', { templateId }),
  duplicateTemplate: (templateId: string) =>
    invoke<TemplateInfo>('duplicate_template', { templateId }),
  deleteTemplate: (templateId: string) => invoke<void>('delete_template', { templateId }),
  importTemplateZip: (zipPath: string) => invoke<TemplateInfo>('import_template_zip', { zipPath }),
  exportTemplateZip: (templateId: string) => invoke<string>('export_template_zip', { templateId }),
  detectProjectType: (path: string) => invoke<ProjectType>('detect_project_type', { path }),
  checkRuntime: (name: string) => invoke<RuntimeInfo>('check_runtime', { name }),
  checkAllRuntimes: () => invoke<RuntimeInfo[]>('check_all_runtimes'),
  projectDoctor: (id: string) => invoke<ProjectDoctorReport>('project_doctor', { id }),
  hostingCompatibilityCheck: (id: string) =>
    invoke<HostingCompatibilityReport>('hosting_compatibility_check', { id }),
  diagnostics: () => invoke<DiagnosticItem[]>('diagnostics'),
};
