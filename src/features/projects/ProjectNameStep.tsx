import type { TFunction } from '../../app/types';
import type { ProjectWizardState } from './wizardTypes';

export function ProjectNameStep({
  value,
  onChange,
  t,
}: {
  value: ProjectWizardState;
  onChange: (value: ProjectWizardState) => void;
  t: TFunction;
}) {
  return (
    <div className="field">
      <span>{t('wizard.projectName')}</span>
      <input
        value={value.name}
        onChange={(event) => onChange({ ...value, name: event.target.value })}
        placeholder="my-local-app"
      />
    </div>
  );
}
