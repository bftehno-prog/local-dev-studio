import { useCallback, useEffect, useMemo, useState } from 'react';
import { AppShell } from './layout/AppShell';
import type { SectionId } from './routes';
import type { TFunction } from './types';
import { DashboardPage } from '../features/dashboard/DashboardPage';
import { DiagnosticsPage } from '../features/diagnostics/DiagnosticsPage';
import { LogsPage } from '../features/logs/LogsPage';
import { OnboardingPage } from '../features/onboarding/OnboardingPage';
import { PortsPage } from '../features/ports/PortsPage';
import { PreviewPanel } from '../features/preview/PreviewPanel';
import { ProjectsPage } from '../features/projects/ProjectsPage';
import { ServersPage } from '../features/servers/ServersPage';
import { SettingsPage } from '../features/settings/SettingsPage';
import { SandboxesPage } from '../features/templates/SandboxesPage';
import { TemplatesPage } from '../features/templates/TemplatesPage';
import { emptySettings } from '../lib/constants';
import { translate } from '../lib/i18n';
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
} from '../lib/types';
import { api, normalizeApiError } from '../shared/lib/api';

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
      <OnboardingPage
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
    <AppShell
      active={active}
      message={message}
      onSelectSection={setActive}
      onRefresh={() => void refreshVisible().catch(showError)}
      onStartAll={() => void run(() => api.startAllProjects(), t('message.allStarted'))}
      onStopAll={() => void run(() => api.stopAllProjects(), t('message.allStopped'))}
      onAddProject={() => setActive('projects')}
      onImportZip={() => setActive('templates')}
      t={t}
      preview={
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
      }
    >
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
        <ProjectsPage
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
        <SettingsPage
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
    </AppShell>
  );
}
