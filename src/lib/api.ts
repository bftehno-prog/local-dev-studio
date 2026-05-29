import { invoke } from '@tauri-apps/api/core';
import type {
  DashboardData,
  LogEntry,
  PortInfo,
  Project,
  ProjectType,
  ServerProcess,
  Settings,
  DiagnosticItem,
  RuntimeInfo,
  TemplateInfo,
} from './types';

export const api = {
  dashboard: () => invoke<DashboardData>('dashboard'),
  listProjects: () => invoke<Project[]>('list_projects'),
  addProject: (path: string) => invoke<Project>('add_project', { path }),
  removeProject: (id: string) => invoke<void>('remove_project', { id }),
  startProject: (id: string) => invoke<Project>('start_project', { id }),
  stopProject: (id: string) => invoke<Project>('stop_project', { id }),
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
  clearCache: (id: string) => invoke<void>('clear_project_cache', { id }),
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
  diagnostics: () => invoke<DiagnosticItem[]>('diagnostics'),
};
