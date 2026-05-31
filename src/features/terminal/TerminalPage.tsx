import { useState } from 'react';
import { Play, ShieldCheck } from 'lucide-react';
import type { TFunction } from '../../app/types';
import type { Project, TerminalRunResult } from '../../lib/types';
import { api } from '../../shared/lib/api';
import { Panel } from '../../components/ui/Panel';

type TerminalPageProps = {
  projects: Project[];
  selectedProjectId?: string;
  onSelectProject: (id: string) => void;
  showError: (error: unknown) => void;
  t: TFunction;
};

const tasks = ['install', 'build', 'test', 'lint'] as const;

export function TerminalPage({
  projects,
  selectedProjectId,
  onSelectProject,
  showError,
  t,
}: TerminalPageProps) {
  const [task, setTask] = useState<(typeof tasks)[number]>('build');
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState<TerminalRunResult | null>(null);
  const selectedProject = projects.find((project) => project.id === selectedProjectId);
  const canRun = Boolean(selectedProjectId && selectedProject?.trusted && !running);

  async function runTask() {
    if (!selectedProjectId) return;
    setRunning(true);
    try {
      setResult(await api.runProjectTask(selectedProjectId, task));
    } catch (error) {
      showError(error);
    } finally {
      setRunning(false);
    }
  }

  return (
    <div className="content">
      <Panel title={t('terminal.title')}>
        <div className="toolbar">
          <select
            value={selectedProjectId || ''}
            onChange={(event) => onSelectProject(event.target.value)}
          >
            <option value="">{t('editor.selectProject')}</option>
            {projects.map((project) => (
              <option key={project.id} value={project.id}>
                {project.name}
              </option>
            ))}
          </select>
          <select value={task} onChange={(event) => setTask(event.target.value as typeof task)}>
            {tasks.map((task) => (
              <option key={task} value={task}>
                {t(`terminal.task.${task}`)}
              </option>
            ))}
          </select>
          <button disabled={!canRun} onClick={() => void runTask()}>
            <Play size={15} /> {running ? t('terminal.running') : t('action.run')}
          </button>
        </div>
        {selectedProject && !selectedProject.trusted && (
          <p className="terminal-note">
            <ShieldCheck size={14} /> {t('terminal.trustRequired')}
          </p>
        )}
      </Panel>
      <Panel title={result?.command || t('terminal.output')}>
        <div className="terminal-meta">
          <span>{result?.cwd || selectedProject?.path || '-'}</span>
          <span>
            {result
              ? result.timed_out
                ? t('terminal.timedOut')
                : `${t('terminal.exitCode')}: ${result.exit_code ?? '-'}`
              : t('terminal.idle')}
          </span>
        </div>
        <pre className="terminal-output">
          {running
            ? t('terminal.running')
            : result
              ? [result.stdout, result.stderr].filter(Boolean).join('\n')
              : t('terminal.placeholder')}
        </pre>
      </Panel>
    </div>
  );
}
