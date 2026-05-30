import { useState } from 'react';
import { Code2, Folder, ListRestart, Play, Plus, RefreshCcw, Square, Trash2 } from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import type { TFunction } from '../../app/types';
import type {
  HostingCompatibilityReport,
  LogEntry,
  Project,
  ProjectDoctorReport,
  ServerProcess,
  Settings,
} from '../../lib/types';
import { api } from '../../shared/lib/api';
import { Panel } from '../../components/ui/Panel';
import { Info, LogList, Status } from '../../shared/ui/DataViews';
import { ProjectWizard } from './ProjectWizard';

export function ProjectsPage({
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
    ['next', 'vite', 'astro', 'node'].includes(project.project_type);

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
      <ProjectWizard
        onCreated={(project) => onRun(async () => project, t('message.projectCreated'))}
        t={t}
      />
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
              <Info
                label={t('settings.packageManager')}
                value={selectedProject.package_manager || settings.package_manager}
              />
              <Info
                label={t('wizard.useDocker')}
                value={selectedProject.use_docker ? 'yes' : 'no'}
              />
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
