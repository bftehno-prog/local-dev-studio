import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  Code2,
  Database,
  Folder,
  ListRestart,
  Play,
  Plus,
  Power,
  RefreshCcw,
  Square,
  Trash2,
  Upload,
} from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import { sections, type SectionId } from './app/routes';
import type { TFunction } from './app/types';
import { DashboardPage } from './features/dashboard/DashboardPage';
import { DiagnosticsPage } from './features/diagnostics/DiagnosticsPage';
import { LogsPage } from './features/logs/LogsPage';
import { PortsPage } from './features/ports/PortsPage';
import { PreviewPanel } from './features/preview/PreviewPanel';
import { ServersPage } from './features/servers/ServersPage';
import { SandboxesPage } from './features/templates/SandboxesPage';
import { TemplatesPage } from './features/templates/TemplatesPage';
import { emptySettings } from './lib/constants';
import { translate, type Language } from './lib/i18n';
import type {
  DashboardData,
  DiagnosticItem,
  LogEntry,
  HostingCompatibilityReport,
  PortInfo,
  Project,
  ProjectDoctorReport,
  RuntimeInfo,
  ServerProcess,
  Settings,
  TemplateInfo,
} from './lib/types';
import { api, normalizeApiError } from './shared/lib/api';
import { Info, LogList, Metric, Status } from './shared/ui/DataViews';

export default function App() {
  const [active, setActive] = useState<SectionId>('dashboard');
  const [dashboard, setDashboard] = useState<DashboardData | null>(null);
  const [projects, setProjects] = useState<Project[]>([]);
  const [servers, setServers] = useState<ServerProcess[]>([]);
  const [ports, setPorts] = useState<PortInfo[]>([]);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [diagnostics, setDiagnostics] = useState<DiagnosticItem[]>([]);
  const [runtimes, setRuntimes] = useState<RuntimeInfo[]>([]);
  const [settings, setSettings] = useState<Settings>(emptySettings);
  const [settingsDirty, setSettingsDirty] = useState(false);
  const [templates, setTemplates] = useState<TemplateInfo[]>([]);
  const [doctorReport, setDoctorReport] = useState<ProjectDoctorReport | null>(null);
  const [hostingReport, setHostingReport] = useState<HostingCompatibilityReport | null>(null);
  const [selectedProjectId, setSelectedProjectId] = useState<string | undefined>();
  const [previewUrl, setPreviewUrl] = useState('');
  const [manualPreviewUrl, setManualPreviewUrl] = useState('');
  const [activePreviewServerId, setActivePreviewServerId] = useState('');
  const [previewKey, setPreviewKey] = useState(0);
  const [fitPreview, setFitPreview] = useState(true);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [previewError, setPreviewError] = useState('');
  const [device, setDevice] = useState('Desktop');
  const [logLevel, setLogLevel] = useState('');
  const [logSearch, setLogSearch] = useState('');
  const [message, setMessage] = useState('');

  const selectedProject = projects.find((project) => project.id === selectedProjectId);
  const language = settings.language || 'en';
  const t: TFunction = (key) => translate(language, key);

  const loadDashboard = useCallback(async () => setDashboard(await api.dashboard()), []);
  const loadProjects = useCallback(async () => {
    const nextProjects = await api.listProjects();
    setProjects(nextProjects);
    setSelectedProjectId((current) => current || nextProjects[0]?.id);
  }, []);
  const loadServersAndPorts = useCallback(async () => {
    const [nextServers, nextPorts] = await Promise.all([api.listServers(), api.listPorts()]);
    setServers(nextServers);
    setPorts(nextPorts);
  }, []);
  const loadLogs = useCallback(async () => {
    setLogs(await api.listLogs(undefined, logLevel || undefined, logSearch || undefined));
  }, [logLevel, logSearch]);
  const loadSettings = useCallback(async () => {
    if (!settingsDirty) {
      setSettings(await api.getSettings());
    }
  }, [settingsDirty]);
  const loadTemplates = useCallback(async () => setTemplates(await api.listTemplates()), []);
  const loadDiagnostics = useCallback(async () => setDiagnostics(await api.diagnostics()), []);
  const loadRuntimes = useCallback(async () => setRuntimes(await api.checkAllRuntimes()), []);
  const refreshVisible = useCallback(async () => {
    const tasks: Promise<unknown>[] = [loadServersAndPorts()];
    if (active === 'dashboard') tasks.push(loadDashboard());
    if (active === 'projects') tasks.push(loadProjects());
    if (active === 'logs') tasks.push(loadLogs());
    if (active === 'templates' || active === 'sandboxes') tasks.push(loadTemplates());
    if (active === 'settings' || active === 'diagnostics') {
      tasks.push(loadSettings(), loadDiagnostics(), loadRuntimes());
    }
    await Promise.all(tasks);
  }, [
    active,
    loadDashboard,
    loadDiagnostics,
    loadLogs,
    loadProjects,
    loadRuntimes,
    loadServersAndPorts,
    loadSettings,
    loadTemplates,
  ]);

  useEffect(() => {
    void Promise.all([
      loadDashboard().catch(showError),
      loadProjects().catch(showError),
      loadServersAndPorts().catch(showError),
      loadLogs().catch(showError),
      loadSettings().catch(showError),
      loadTemplates().catch(showError),
      loadRuntimes().catch(showError),
    ]);
  }, [
    loadDashboard,
    loadLogs,
    loadProjects,
    loadRuntimes,
    loadServersAndPorts,
    loadSettings,
    loadTemplates,
  ]);

  useEffect(() => {
    if (active === 'settings' || active === 'diagnostics') {
      void loadDiagnostics().catch(showError);
      void loadRuntimes().catch(showError);
    }
  }, [active, loadDiagnostics, loadRuntimes]);

  useEffect(() => {
    const timer = window.setInterval(() => void loadServersAndPorts().catch(showError), 2000);
    return () => window.clearInterval(timer);
  }, [loadServersAndPorts]);

  useEffect(() => {
    const timer = window.setInterval(() => void loadDashboard().catch(showError), 5000);
    return () => window.clearInterval(timer);
  }, [loadDashboard]);

  useEffect(() => {
    if (active !== 'logs') return;
    const timer = window.setInterval(() => void loadLogs().catch(showError), 5000);
    return () => window.clearInterval(timer);
  }, [active, loadLogs]);

  useEffect(() => {
    const running = selectedProject
      ? servers.find((server) => server.project_id === selectedProject.id)
      : servers[0];
    const activeServer =
      servers.find((server) => server.project_id === activePreviewServerId) ?? running;
    if (!manualPreviewUrl) {
      setPreviewUrl(activeServer?.url ?? '');
      setActivePreviewServerId(activeServer?.project_id ?? '');
    }
  }, [activePreviewServerId, manualPreviewUrl, selectedProject, servers]);

  function showError(error: unknown) {
    setMessage(normalizeApiError(error).message);
  }

  async function run(action: () => Promise<unknown>, success: string) {
    try {
      setMessage('');
      await action();
      setMessage(success);
      await refreshVisible();
    } catch (error) {
      showError(error);
    }
  }

  function updateSettings(nextSettings: Settings) {
    setSettingsDirty(true);
    setSettings(nextSettings);
  }

  async function saveSettings() {
    await run(async () => {
      const saved = await api.saveSettings(settings);
      setSettings(saved);
      setSettingsDirty(false);
    }, t('message.settingsSaved'));
  }

  async function refreshDiagnostics() {
    await run(loadDiagnostics, t('message.diagnosticsRefreshed'));
  }

  async function refreshRuntimes() {
    await run(loadRuntimes, t('message.runtimesRefreshed'));
  }

  async function copyDiagnosticsReport() {
    const report = [
      '# Local Dev Studio Diagnostics',
      '',
      `Generated: ${new Date().toISOString()}`,
      '',
      '## Runtimes',
      ...runtimes.map(
        (runtime) =>
          `- ${runtime.name}: ${runtime.found ? 'OK' : 'Missing'} | ${runtime.version || '-'} | ${runtime.source} | ${runtime.path || '-'}`,
      ),
      '',
      '## Diagnostics',
      ...diagnostics.map(
        (item) =>
          `- ${item.name}: ${item.status} | ${item.version || '-'} | ${item.path || '-'} | ${item.error || '-'}`,
      ),
    ].join('\n');
    await navigator.clipboard.writeText(report);
    setMessage(t('diagnostics.reportCopied'));
  }

  async function finishOnboarding() {
    const nextSettings = { ...settings, onboarding_completed: true };
    const saved = await api.saveSettings(nextSettings);
    setSettings(saved);
    setSettingsDirty(false);
    setActive('dashboard');
    setMessage(t('onboarding.done'));
  }

  const previewWidth = useMemo(() => {
    if (device === 'Desktop') return settings.desktop_width;
    if (device === 'Laptop') return settings.laptop_width;
    if (device === 'Tablet') return settings.tablet_width;
    if (device === 'Mobile') return settings.mobile_width;
    return settings.custom_width;
  }, [device, settings]);
  const activePreviewServer = servers.find((server) => server.project_id === activePreviewServerId);
  const localPreviewUrl = manualPreviewUrl || previewUrl;
  const networkPreviewUrl = settings.enable_network_preview
    ? activePreviewServer?.network_url || localPreviewUrl.replace('localhost', '127.0.0.1')
    : localPreviewUrl;
  const previewScale = fitPreview ? Math.min(1, 620 / previewWidth) : 1;

  useEffect(() => {
    if (localPreviewUrl) {
      setPreviewLoading(true);
      setPreviewError('');
    }
  }, [localPreviewUrl, previewKey, device]);

  if (!settings.onboarding_completed) {
    return (
      <OnboardingView
        diagnostics={diagnostics}
        runtimes={runtimes}
        settings={settings}
        onRefresh={() => void Promise.all([loadDiagnostics(), loadRuntimes()]).catch(showError)}
        onFinish={() => void finishOnboarding().catch(showError)}
        onOpenSettings={() => {
          setSettings((current) => ({ ...current, onboarding_completed: true }));
          setActive('settings');
        }}
        t={t}
      />
    );
  }

  return (
    <main className="app">
      <aside className="sidebar">
        <div className="brand">
          <Database size={22} />
          <div>
            <strong>Local Dev Studio</strong>
            <span>Windows</span>
          </div>
        </div>
        <nav>
          {sections.map(([id, labelKey, Icon]) => (
            <button
              className={active === id ? 'nav-item active' : 'nav-item'}
              key={id}
              onClick={() => setActive(id)}
            >
              <Icon size={17} />
              {t(labelKey)}
            </button>
          ))}
        </nav>
      </aside>

      <section className="workspace">
        <header className="topbar">
          <div>
            <h1>{t(sections.find(([id]) => id === active)?.[1] ?? 'nav.dashboard')}</h1>
            <p>{message || t('top.subtitle')}</p>
          </div>
          <div className="top-actions">
            <button onClick={() => void refreshVisible().catch(showError)}>
              <RefreshCcw size={16} /> {t('action.refresh')}
            </button>
            <button onClick={() => void run(() => api.startAllProjects(), t('message.allStarted'))}>
              <Play size={16} /> {t('action.startAll')}
            </button>
            <button onClick={() => void run(() => api.stopAllProjects(), t('message.allStopped'))}>
              <Power size={16} /> {t('action.stopAll')}
            </button>
            <button onClick={() => setActive('projects')}>
              <Plus size={16} /> {t('action.addProject')}
            </button>
            <button onClick={() => setActive('templates')}>
              <Upload size={16} /> {t('action.importZip')}
            </button>
          </div>
        </header>

        {active === 'dashboard' && (
          <DashboardPage
            dashboard={dashboard}
            servers={servers}
            ports={ports}
            projects={projects}
            t={t}
          />
        )}
        {active === 'projects' && (
          <ProjectsView
            projects={projects}
            servers={servers}
            logs={logs}
            settings={settings}
            doctorReport={doctorReport}
            hostingReport={hostingReport}
            selectedProjectId={selectedProjectId}
            onSelect={setSelectedProjectId}
            onDoctor={setDoctorReport}
            onHosting={setHostingReport}
            onRun={run}
            t={t}
          />
        )}
        {active === 'sandboxes' && <SandboxesPage templates={templates} onRun={run} t={t} />}
        {active === 'templates' && <TemplatesPage templates={templates} onRun={run} t={t} />}
        {active === 'servers' && <ServersPage servers={servers} onRun={run} t={t} />}
        {active === 'ports' && <PortsPage ports={ports} onRun={run} t={t} />}
        {active === 'logs' && (
          <LogsPage
            logs={logs}
            level={logLevel}
            search={logSearch}
            onLevel={setLogLevel}
            onSearch={setLogSearch}
            onRun={run}
            t={t}
          />
        )}
        {active === 'diagnostics' && (
          <DiagnosticsPage
            diagnostics={diagnostics}
            runtimes={runtimes}
            onRefresh={() => void Promise.all([refreshDiagnostics(), refreshRuntimes()])}
            onCopyReport={() => void copyDiagnosticsReport().catch(showError)}
            onOpenLogs={() => setActive('logs')}
            t={t}
          />
        )}
        {active === 'settings' && (
          <SettingsView
            settings={settings}
            onChange={updateSettings}
            onSave={saveSettings}
            diagnostics={diagnostics}
            runtimes={runtimes}
            onRefreshDiagnostics={refreshDiagnostics}
            onRefreshRuntimes={refreshRuntimes}
            t={t}
            language={language}
          />
        )}
      </section>

      <PreviewPanel
        t={t}
        settings={settings}
        servers={servers}
        selectedProject={selectedProject}
        activePreviewServerId={activePreviewServerId}
        setActivePreviewServerId={setActivePreviewServerId}
        previewUrl={previewUrl}
        setPreviewUrl={setPreviewUrl}
        manualPreviewUrl={manualPreviewUrl}
        setManualPreviewUrl={setManualPreviewUrl}
        previewKey={previewKey}
        setPreviewKey={setPreviewKey}
        fitPreview={fitPreview}
        setFitPreview={setFitPreview}
        previewLoading={previewLoading}
        setPreviewLoading={setPreviewLoading}
        previewError={previewError}
        setPreviewError={setPreviewError}
        device={device}
        setDevice={setDevice}
        previewWidth={previewWidth}
        previewScale={previewScale}
        localPreviewUrl={localPreviewUrl}
        networkPreviewUrl={networkPreviewUrl}
        activePreviewServer={activePreviewServer}
        showError={showError}
        run={run}
      />
    </main>
  );
}

function OnboardingView({
  diagnostics,
  runtimes,
  settings,
  onRefresh,
  onFinish,
  onOpenSettings,
  t,
}: {
  diagnostics: DiagnosticItem[];
  runtimes: RuntimeInfo[];
  settings: Settings;
  onRefresh: () => void;
  onFinish: () => void;
  onOpenSettings: () => void;
  t: TFunction;
}) {
  const runtimeReady = runtimes.filter((runtime) => runtime.found).length;
  const projectsFolderOk =
    diagnostics.find((item) => item.name === 'Projects folder')?.status === 'OK';
  const sandboxesFolderOk =
    diagnostics.find((item) => item.name === 'Sandboxes folder')?.status === 'OK';
  return (
    <main className="onboarding">
      <section className="onboarding-shell">
        <div className="brand">
          <Database size={22} />
          <div>
            <strong>Local Dev Studio</strong>
            <span>Windows</span>
          </div>
        </div>
        <header>
          <h1>{t('onboarding.title')}</h1>
          <p>{t('onboarding.subtitle')}</p>
        </header>
        <div className="grid two">
          <Panel title={t('onboarding.environment')}>
            <Metric
              label={t('settings.runtimeHealth')}
              value={`${runtimeReady}/${runtimes.length}`}
            />
            <div className="logs">
              {runtimes.map((runtime) => (
                <div className={`log ${runtime.found ? 'info' : 'warning'}`} key={runtime.name}>
                  <strong>{runtime.name}</strong>
                  <p>
                    {runtime.found ? runtime.version || 'OK' : runtime.error || t('empty.notFound')}
                  </p>
                </div>
              ))}
            </div>
          </Panel>
          <Panel title={t('onboarding.folders')}>
            <Info label={t('settings.projectsFolder')} value={settings.projects_folder || '-'} />
            <Info label={t('settings.sandboxesFolder')} value={settings.sandboxes_folder || '-'} />
            <Info label="Projects OK" value={projectsFolderOk ? 'OK' : t('empty.notFound')} />
            <Info label="Sandboxes OK" value={sandboxesFolderOk ? 'OK' : t('empty.notFound')} />
          </Panel>
          <Panel title={t('onboarding.ports')}>
            <Info label={t('settings.portStart')} value={String(settings.port_start)} />
            <Info label={t('settings.portEnd')} value={String(settings.port_end)} />
            <Info label={t('settings.processTimeout')} value={`${settings.process_timeout}s`} />
          </Panel>
          <Panel title={t('onboarding.preview')}>
            <Info
              label={t('settings.networkPreview')}
              value={settings.enable_network_preview ? 'ON' : 'OFF'}
            />
            <Info
              label={t('settings.openExternalBrowser')}
              value={settings.open_external_browser_on_start ? 'ON' : 'OFF'}
            />
            <Info label={t('settings.defaultDevice')} value={settings.default_device} />
          </Panel>
        </div>
        <div className="top-actions">
          <button onClick={onRefresh}>
            <RefreshCcw size={16} /> {t('action.rerunDiagnostics')}
          </button>
          <button onClick={onOpenSettings}>{t('action.openSettings')}</button>
          <button onClick={onFinish} className="primary-save">
            {t('action.finishOnboarding')}
          </button>
        </div>
      </section>
    </main>
  );
}

function ProjectsView({
  projects,
  servers,
  logs,
  settings,
  doctorReport,
  hostingReport,
  selectedProjectId,
  onSelect,
  onDoctor,
  onHosting,
  onRun,
  t,
}: {
  projects: Project[];
  servers: ServerProcess[];
  logs: LogEntry[];
  settings: Settings;
  doctorReport: ProjectDoctorReport | null;
  hostingReport: HostingCompatibilityReport | null;
  selectedProjectId?: string;
  onSelect: (id: string) => void;
  onDoctor: (report: ProjectDoctorReport) => void;
  onHosting: (report: HostingCompatibilityReport) => void;
  onRun: (action: () => Promise<unknown>, success: string) => Promise<void>;
  t: TFunction;
}) {
  const [manualPath, setManualPath] = useState('');
  const selectedProject = projects.find((project) => project.id === selectedProjectId);
  const selectedServer = selectedProject
    ? servers.find((server) => server.project_id === selectedProject.id)
    : undefined;
  const selectedLogs = selectedProject
    ? logs.filter((log) => log.project_id === selectedProject.id).slice(0, 5)
    : [];
  const isNodeProject = (project: Project) =>
    ['next', 'vite', 'astro'].includes(project.project_type);
  async function chooseFolder() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t('projects.selectFolder'),
    });
    if (typeof selected === 'string') {
      setManualPath(selected);
    }
  }
  return (
    <div className="content">
      <Panel title={t('projects.addExisting')}>
        <div className="input-row">
          <input
            value={manualPath}
            onChange={(event) => setManualPath(event.target.value)}
            placeholder="C:\Users\User\Projects\my-next-app"
          />
          <button onClick={() => void chooseFolder()}>
            <Folder size={16} /> {t('action.chooseFolder')}
          </button>
          <button
            onClick={() => onRun(() => api.addProject(manualPath), t('message.projectAdded'))}
          >
            <Plus size={16} /> {t('action.add')}
          </button>
        </div>
      </Panel>
      <Panel title={t('projects.title')}>
        <table>
          <thead>
            <tr>
              <th>{t('table.name')}</th>
              <th>{t('table.type')}</th>
              <th>{t('table.port')}</th>
              <th>{t('table.status')}</th>
              <th>{t('table.trust')}</th>
              <th>{t('table.path')}</th>
              <th>{t('table.actions')}</th>
            </tr>
          </thead>
          <tbody>
            {projects.map((project) => (
              <tr
                className={project.id === selectedProjectId ? 'selected' : ''}
                key={project.id}
                onClick={() => onSelect(project.id)}
              >
                <td>{project.name}</td>
                <td>{project.project_type}</td>
                <td>{project.port || '-'}</td>
                <td>
                  <Status status={project.status} t={t} />
                </td>
                <td>
                  <span className={`status ${project.trusted ? 'running' : 'warning'}`}>
                    {project.trusted ? 'trusted' : 'untrusted'}
                  </span>
                </td>
                <td>
                  <code>{project.path}</code>
                </td>
                <td className="actions">
                  <button
                    onClick={() =>
                      onRun(() => api.startProject(project.id), t('message.projectStarted'))
                    }
                  >
                    <Play size={15} />
                  </button>
                  <button
                    onClick={() =>
                      onRun(() => api.stopProject(project.id), t('message.projectStopped'))
                    }
                  >
                    <Square size={15} />
                  </button>
                  <button
                    onClick={() =>
                      onRun(() => api.restartProject(project.id), t('message.projectRestarted'))
                    }
                  >
                    <ListRestart size={15} />
                  </button>
                  <button
                    onClick={() =>
                      onRun(() => api.openPath(project.path), t('message.folderOpened'))
                    }
                  >
                    <Folder size={15} />
                  </button>
                  <button
                    onClick={() =>
                      onRun(() => api.openInCode(project.path), t('message.codeOpened'))
                    }
                  >
                    <Code2 size={15} />
                  </button>
                  <button
                    onClick={() =>
                      onRun(() => api.clearCache(project.id), t('message.cacheCleared'))
                    }
                  >
                    <RefreshCcw size={15} />
                  </button>
                  {isNodeProject(project) && (
                    <button
                      onClick={() =>
                        onRun(
                          () => api.installProjectDependencies(project.id),
                          t('message.dependenciesInstalling'),
                        )
                      }
                      disabled={!project.trusted || project.status === 'installing'}
                    >
                      {t('action.installDependencies')}
                    </button>
                  )}
                  <button
                    onClick={() =>
                      onRun(
                        async () => onDoctor(await api.projectDoctor(project.id)),
                        t('message.doctorReady'),
                      )
                    }
                  >
                    {t('action.runDoctor')}
                  </button>
                  <button
                    onClick={() =>
                      onRun(() => api.trustProject(project.id), t('message.projectTrusted'))
                    }
                    disabled={project.trusted}
                  >
                    {t('action.trustProject')}
                  </button>
                  <button
                    onClick={() =>
                      onRun(() => api.resetProjectTrust(project.id), t('message.projectTrustReset'))
                    }
                    disabled={!project.trusted}
                  >
                    {t('action.resetTrust')}
                  </button>
                  <button
                    onClick={() =>
                      onRun(() => api.removeProject(project.id), t('message.projectRemoved'))
                    }
                  >
                    <Trash2 size={15} />
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </Panel>
      {doctorReport && (
        <Panel title={`${t('projects.doctor')}: ${doctorReport.project_name}`}>
          <div className="logs">
            {doctorReport.checks.map((check) => (
              <div
                className={`log ${check.status === 'ok' ? 'info' : 'warning'}`}
                key={check.label}
              >
                <strong>{check.status === 'ok' ? 'OK' : 'WARN'}</strong>
                <p>
                  {check.label}: {check.message}
                </p>
              </div>
            ))}
          </div>
        </Panel>
      )}
      {selectedProject && (
        <Panel title={`${t('projects.detail')}: ${selectedProject.name}`}>
          <div className="grid two">
            <div>
              <Info label={t('table.type')} value={selectedProject.project_type} />
              <Info label={t('table.status')} value={selectedProject.status} />
              <Info
                label={t('table.trust')}
                value={selectedProject.trusted ? 'trusted' : 'untrusted'}
              />
              <Info label={t('settings.packageManager')} value={settings.package_manager} />
              <Info label={t('table.runtime')} value={selectedProject.trusted_runtime || '-'} />
              <Info
                label={t('table.port')}
                value={String(selectedProject.port || selectedServer?.port || '-')}
              />
              <Info label={t('projects.previewUrl')} value={selectedServer?.url || '-'} />
              <Info
                label={t('table.command')}
                value={selectedProject.command || selectedServer?.command || '-'}
              />
              <Info label={t('table.path')} value={selectedProject.path} />
            </div>
            <div className="toolbar project-detail-actions">
              <button
                onClick={() =>
                  onRun(() => api.startProject(selectedProject.id), t('message.projectStarted'))
                }
              >
                <Play size={15} /> {t('action.start')}
              </button>
              <button
                onClick={() =>
                  onRun(() => api.stopProject(selectedProject.id), t('message.projectStopped'))
                }
              >
                <Square size={15} /> {t('status.stopped')}
              </button>
              <button
                onClick={() =>
                  onRun(() => api.restartProject(selectedProject.id), t('message.projectRestarted'))
                }
              >
                <ListRestart size={15} /> {t('message.projectRestarted')}
              </button>
              <button
                disabled={!selectedServer?.url}
                onClick={() =>
                  selectedServer?.url &&
                  onRun(() => api.openExternal(selectedServer.url), t('preview.openBrowser'))
                }
              >
                {t('preview.openBrowser')}
              </button>
              <button
                onClick={() =>
                  onRun(() => api.openPath(selectedProject.path), t('message.folderOpened'))
                }
              >
                <Folder size={15} /> {t('message.folderOpened')}
              </button>
              <button
                onClick={() =>
                  onRun(() => api.openInCode(selectedProject.path), t('message.codeOpened'))
                }
              >
                <Code2 size={15} /> {t('message.codeOpened')}
              </button>
              <button
                onClick={() =>
                  onRun(() => api.clearCache(selectedProject.id), t('message.cacheCleared'))
                }
              >
                <RefreshCcw size={15} /> {t('message.cacheCleared')}
              </button>
              {isNodeProject(selectedProject) && (
                <button
                  onClick={() =>
                    onRun(
                      () => api.installProjectDependencies(selectedProject.id),
                      t('message.dependenciesInstalling'),
                    )
                  }
                  disabled={!selectedProject.trusted || selectedProject.status === 'installing'}
                >
                  {t('action.installDependencies')}
                </button>
              )}
              <button
                onClick={() =>
                  onRun(
                    async () => onDoctor(await api.projectDoctor(selectedProject.id)),
                    t('message.doctorReady'),
                  )
                }
              >
                {t('action.runDoctor')}
              </button>
              <button
                onClick={() =>
                  onRun(
                    async () => onHosting(await api.hostingCompatibilityCheck(selectedProject.id)),
                    t('message.hostingReady'),
                  )
                }
                disabled={!['php', 'static'].includes(selectedProject.project_type)}
              >
                {t('action.checkHosting')}
              </button>
            </div>
          </div>
          <h2>{t('projects.recentLogs')}</h2>
          <LogList logs={selectedLogs} t={t} />
        </Panel>
      )}
      {hostingReport && (
        <Panel title={`${t('projects.hostingCompatibility')}: ${hostingReport.project_name}`}>
          <div className="logs">
            {hostingReport.checks.map((check) => (
              <div
                className={`log ${check.status === 'ok' ? 'info' : 'warning'}`}
                key={check.label}
              >
                <strong>{check.status === 'ok' ? 'OK' : 'WARN'}</strong>
                <p>
                  {check.label}: {check.message}
                </p>
              </div>
            ))}
          </div>
        </Panel>
      )}
    </div>
  );
}

function SettingsView({
  settings,
  onChange,
  onSave,
  diagnostics,
  runtimes,
  onRefreshDiagnostics,
  onRefreshRuntimes,
  t,
  language,
}: {
  settings: Settings;
  onChange: (settings: Settings) => void;
  onSave: () => Promise<void>;
  diagnostics: DiagnosticItem[];
  runtimes: RuntimeInfo[];
  onRefreshDiagnostics: () => Promise<void>;
  onRefreshRuntimes: () => Promise<void>;
  t: TFunction;
  language: Language;
}) {
  const [settingsTab, setSettingsTab] = useState('general');
  const set = <K extends keyof Settings>(key: K, value: Settings[K]) =>
    onChange({ ...settings, [key]: value });
  const tabs = [
    ['general', t('settings.general')],
    ['runtime', t('settings.runtime')],
    ['servers', t('settings.servers')],
    ['preview', t('settings.preview')],
    ['next', t('settings.next')],
    ['experimental', t('settings.experimental')],
    ['diagnostics', t('settings.diagnostics')],
  ];
  return (
    <div className="content">
      <div className="settings-tabs" role="tablist" aria-label={t('nav.settings')}>
        {tabs.map(([id, label]) => (
          <button
            key={id}
            className={settingsTab === id ? 'active' : ''}
            role="tab"
            aria-selected={settingsTab === id}
            onClick={() => setSettingsTab(id)}
          >
            {label}
          </button>
        ))}
      </div>

      {settingsTab === 'general' && (
        <Panel title={t('settings.general')}>
          <label className="field">
            <span>{t('settings.language')}</span>
            <select
              value={language}
              onChange={(event) => set('language', event.target.value as Language)}
            >
              <option value="en">English</option>
              <option value="ru">Русский</option>
            </select>
          </label>
          <Field
            label={t('settings.projectsFolder')}
            value={settings.projects_folder}
            onChange={(value) => set('projects_folder', value)}
          />
          <Field
            label={t('settings.sandboxesFolder')}
            value={settings.sandboxes_folder}
            onChange={(value) => set('sandboxes_folder', value)}
          />
          <Toggle
            label={t('settings.openPreview')}
            checked={settings.open_preview_automatically}
            onChange={(value) => set('open_preview_automatically', value)}
            disabled
          />
          <Toggle
            label={t('settings.startMinimized')}
            checked={settings.start_minimized}
            onChange={(value) => set('start_minimized', value)}
          />
          <Toggle
            label={t('settings.launchOnStartup')}
            checked={settings.launch_on_startup}
            onChange={(value) => set('launch_on_startup', value)}
          />
        </Panel>
      )}

      {settingsTab === 'runtime' && (
        <Panel title={t('settings.runtime')}>
          <div className="toolbar">
            <button onClick={() => void onRefreshRuntimes()}>
              <RefreshCcw size={16} /> {t('action.rerunDiagnostics')}
            </button>
          </div>
          <Toggle
            label={t('settings.useBundledNode')}
            checked={settings.use_bundled_node}
            onChange={(value) => set('use_bundled_node', value)}
          />
          <Field
            label={t('settings.packageManager')}
            value={settings.package_manager}
            onChange={(value) => set('package_manager', value)}
          />
          <Field
            label={t('settings.nodePath')}
            value={settings.node_path}
            onChange={(value) => set('node_path', value)}
          />
          <Field
            label={t('settings.npmPath')}
            value={settings.npm_path}
            onChange={(value) => set('npm_path', value)}
          />
          <Field
            label={t('settings.pnpmPath')}
            value={settings.pnpm_path}
            onChange={(value) => set('pnpm_path', value)}
          />
          <Field
            label={t('settings.yarnPath')}
            value={settings.yarn_path}
            onChange={(value) => set('yarn_path', value)}
          />
          <Field
            label={t('settings.bunPath')}
            value={settings.bun_path}
            onChange={(value) => set('bun_path', value)}
          />
          <Field
            label={t('settings.phpPath')}
            value={settings.php_path}
            onChange={(value) => set('php_path', value)}
          />
          <Field
            label={t('settings.gitPath')}
            value={settings.git_path}
            onChange={(value) => set('git_path', value)}
          />
          <h2>{t('settings.runtimeHealth')}</h2>
          <table>
            <thead>
              <tr>
                <th>{t('table.runtime')}</th>
                <th>{t('table.status')}</th>
                <th>{t('table.version')}</th>
                <th>{t('settings.source')}</th>
                <th>{t('table.path')}</th>
                <th>{t('table.error')}</th>
              </tr>
            </thead>
            <tbody>
              {runtimes.map((runtime) => (
                <tr key={runtime.name}>
                  <td>{runtime.name}</td>
                  <td>
                    <span className={`status ${runtime.found ? 'running' : 'error'}`}>
                      {runtime.found ? 'OK' : t('empty.notFound')}
                    </span>
                  </td>
                  <td>
                    <code>{runtime.version || '-'}</code>
                  </td>
                  <td>{runtime.source}</td>
                  <td>
                    <code>{runtime.path || '-'}</code>
                  </td>
                  <td>{runtime.error || '-'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </Panel>
      )}

      {settingsTab === 'servers' && (
        <Panel title={t('settings.servers')}>
          <NumberField
            label={t('settings.portStart')}
            value={settings.port_start}
            onChange={(value) => set('port_start', value)}
          />
          <NumberField
            label={t('settings.portEnd')}
            value={settings.port_end}
            onChange={(value) => set('port_end', value)}
          />
          <NumberField
            label={t('settings.processTimeout')}
            value={settings.process_timeout}
            onChange={(value) => set('process_timeout', value)}
          />
          <NumberField
            label={t('settings.logRetention')}
            value={settings.log_retention}
            onChange={(value) => set('log_retention', value)}
          />
          <Field
            label={t('settings.envVars')}
            value={settings.environment_variables}
            onChange={(value) => set('environment_variables', value)}
          />
        </Panel>
      )}

      {settingsTab === 'preview' && (
        <Panel title={t('settings.preview')}>
          <Field
            label={t('settings.defaultDevice')}
            value={settings.default_device}
            onChange={(value) => set('default_device', value)}
          />
          <NumberField
            label={t('settings.customWidth')}
            value={settings.custom_width}
            onChange={(value) => set('custom_width', value)}
          />
          <Toggle
            label={t('settings.networkPreview')}
            checked={settings.enable_network_preview}
            onChange={(value) => set('enable_network_preview', value)}
          />
          <Toggle
            label={t('settings.autoReloadPreview')}
            checked={settings.auto_reload_preview}
            onChange={(value) => set('auto_reload_preview', value)}
          />
          <Toggle
            label={t('settings.openExternalBrowser')}
            checked={settings.open_external_browser_on_start}
            onChange={(value) => set('open_external_browser_on_start', value)}
          />
        </Panel>
      )}

      {settingsTab === 'next' && (
        <Panel title={t('settings.next')}>
          <Toggle
            label={t('settings.useTurbopack')}
            checked={settings.use_turbopack}
            onChange={(value) => set('use_turbopack', value)}
          />
          <Toggle
            label={t('settings.clearNext')}
            checked={settings.clear_next_before_start}
            onChange={(value) => set('clear_next_before_start', value)}
          />
          <NumberField
            label={t('settings.defaultNextPort')}
            value={settings.default_next_port}
            onChange={(value) => set('default_next_port', value)}
          />
        </Panel>
      )}

      {settingsTab === 'experimental' && (
        <Panel title={t('settings.experimental')}>
          <Toggle
            label={`${t('settings.https')} (${t('status.inDevelopment')})`}
            checked={settings.enable_https}
            onChange={(value) => set('enable_https', value)}
            disabled
          />
          <Field
            label={`${t('settings.proxyRules')} (${t('status.inDevelopment')})`}
            value={settings.proxy_rules}
            onChange={(value) => set('proxy_rules', value)}
            disabled
          />
          <Field
            label={`${t('settings.hosts')} (${t('status.inDevelopment')})`}
            value={settings.hosts}
            onChange={(value) => set('hosts', value)}
            disabled
          />
          <Field
            label={`${t('settings.sslCertificates')} (${t('status.inDevelopment')})`}
            value={settings.ssl_certificates}
            onChange={(value) => set('ssl_certificates', value)}
            disabled
          />
        </Panel>
      )}

      {settingsTab === 'diagnostics' && (
        <Panel title={t('settings.diagnostics')}>
          <div className="toolbar">
            <button onClick={() => void onRefreshDiagnostics()}>
              <RefreshCcw size={16} /> {t('action.rerunDiagnostics')}
            </button>
          </div>
          <table>
            <thead>
              <tr>
                <th>{t('table.name')}</th>
                <th>{t('table.status')}</th>
                <th>{t('table.version')}</th>
                <th>{t('table.path')}</th>
                <th>{t('table.error')}</th>
              </tr>
            </thead>
            <tbody>
              {diagnostics.map((item) => (
                <tr key={`${item.name}-${item.path}`}>
                  <td>{item.name}</td>
                  <td>
                    <span className={`status ${item.status.toLowerCase()}`}>{item.status}</span>
                  </td>
                  <td>
                    <code>{item.version || '-'}</code>
                  </td>
                  <td>
                    <code>{item.path || '-'}</code>
                  </td>
                  <td>{item.error || '-'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </Panel>
      )}
      <button className="primary-save" onClick={() => void onSave()}>
        {t('action.saveSettings')}
      </button>
    </div>
  );
}

function Panel({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="panel">
      <h2>{title}</h2>
      {children}
    </section>
  );
}

function Field({
  label,
  value,
  onChange,
  disabled = false,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
}) {
  return (
    <label className="field">
      <span>{label}</span>
      <input value={value} disabled={disabled} onChange={(event) => onChange(event.target.value)} />
    </label>
  );
}

function NumberField({
  label,
  value,
  onChange,
}: {
  label: string;
  value: number;
  onChange: (value: number) => void;
}) {
  return (
    <label className="field">
      <span>{label}</span>
      <input
        type="number"
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
      />
    </label>
  );
}

function Toggle({
  label,
  checked,
  onChange,
  disabled = false,
}: {
  label: string;
  checked: boolean;
  onChange: (value: boolean) => void;
  disabled?: boolean;
}) {
  return (
    <label className="toggle">
      <input
        type="checkbox"
        checked={checked}
        disabled={disabled}
        onChange={(event) => onChange(event.target.checked)}
      />{' '}
      <span>{label}</span>
    </label>
  );
}
