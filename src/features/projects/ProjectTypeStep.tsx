import type { TFunction } from '../../app/types';
import { PROJECT_TYPES, type ProjectWizardState } from './wizardTypes';

export function ProjectTypeStep({
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
      <span>{t('wizard.projectType')}</span>
      <select
        value={value.project_type}
        onChange={(event) => onChange({ ...value, project_type: event.target.value })}
      >
        {PROJECT_TYPES.map((type) => (
          <option key={type.value} value={type.value}>
            {type.label}
          </option>
        ))}
      </select>
    </div>
  );
}
