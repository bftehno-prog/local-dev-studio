import { useState } from 'react';
import { Plus } from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import type { TFunction } from '../../app/types';
import type { Project } from '../../lib/types';
import { api, normalizeApiError } from '../../shared/lib/api';
import { Panel } from '../../components/ui/Panel';
import { ProjectCreateProgress } from './ProjectCreateProgress';
import { ProjectNameStep } from './ProjectNameStep';
import { ProjectOptionsStep } from './ProjectOptionsStep';
import { ProjectPathStep } from './ProjectPathStep';
import { ProjectTypeStep } from './ProjectTypeStep';
import { initialProjectWizardState, type ProjectWizardState } from './wizardTypes';

export function ProjectWizard({
  onCreated,
  t,
}: {
  onCreated: (project: Project) => void;
  t: TFunction;
}) {
  const [value, setValue] = useState<ProjectWizardState>(initialProjectWizardState);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');

  async function chooseBaseFolder() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t('projects.selectFolder'),
    });
    if (typeof selected === 'string') {
      setValue((current) => ({ ...current, path: selected }));
    }
  }

  async function createProject() {
    setBusy(true);
    setError('');
    try {
      const project = await api.createProject(value);
      onCreated(project);
      setValue(initialProjectWizardState);
    } catch (caught) {
      setError(normalizeApiError(caught).message);
    } finally {
      setBusy(false);
    }
  }

  const canCreate = value.name.trim().length > 0 && value.path.trim().length > 0 && !busy;

  return (
    <Panel title={t('wizard.title')}>
      <div className="wizard-grid">
        <ProjectTypeStep value={value} onChange={setValue} t={t} />
        <ProjectNameStep value={value} onChange={setValue} t={t} />
        <ProjectPathStep
          value={value}
          onChange={setValue}
          onChooseFolder={chooseBaseFolder}
          t={t}
        />
        <ProjectOptionsStep value={value} onChange={setValue} t={t} />
      </div>
      <div className="toolbar">
        <button disabled={!canCreate} onClick={() => void createProject()}>
          <Plus size={16} /> {t('action.create')}
        </button>
        <ProjectCreateProgress busy={busy} error={error} t={t} />
      </div>
    </Panel>
  );
}
