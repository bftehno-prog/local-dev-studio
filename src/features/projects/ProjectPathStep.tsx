import { Folder } from 'lucide-react';
import type { TFunction } from '../../app/types';
import type { ProjectWizardState } from './wizardTypes';

export function ProjectPathStep({
  value,
  onChange,
  onChooseFolder,
  t,
}: {
  value: ProjectWizardState;
  onChange: (value: ProjectWizardState) => void;
  onChooseFolder: () => Promise<void>;
  t: TFunction;
}) {
  return (
    <div className="field">
      <span>{t('wizard.projectPath')}</span>
      <div className="input-row">
        <input
          value={value.path}
          onChange={(event) => onChange({ ...value, path: event.target.value })}
          placeholder="C:\Users\User\Projects"
        />
        <button onClick={() => void onChooseFolder()}>
          <Folder size={16} /> {t('action.chooseFolder')}
        </button>
      </div>
    </div>
  );
}
