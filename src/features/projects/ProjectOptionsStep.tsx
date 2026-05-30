import type { TFunction } from '../../app/types';
import { PACKAGE_MANAGERS, type ProjectWizardState } from './wizardTypes';

export function ProjectOptionsStep({
  value,
  onChange,
  t,
}: {
  value: ProjectWizardState;
  onChange: (value: ProjectWizardState) => void;
  t: TFunction;
}) {
  return (
    <div className="grid two">
      <div className="field">
        <span>{t('settings.packageManager')}</span>
        <select
          value={value.package_manager || 'pnpm'}
          onChange={(event) => onChange({ ...value, package_manager: event.target.value })}
        >
          {PACKAGE_MANAGERS.map((manager) => (
            <option key={manager} value={manager}>
              {manager}
            </option>
          ))}
        </select>
      </div>
      <div>
        <label className="toggle">
          <input
            type="checkbox"
            checked={Boolean(value.auto_install)}
            onChange={(event) => onChange({ ...value, auto_install: event.target.checked })}
          />
          <span>{t('wizard.autoInstall')}</span>
        </label>
        <label className="toggle">
          <input
            type="checkbox"
            checked={Boolean(value.auto_start)}
            onChange={(event) => onChange({ ...value, auto_start: event.target.checked })}
          />
          <span>{t('wizard.autoStart')}</span>
        </label>
        <label className="toggle">
          <input
            type="checkbox"
            checked={Boolean(value.use_docker)}
            onChange={(event) => onChange({ ...value, use_docker: event.target.checked })}
          />
          <span>{t('wizard.useDocker')}</span>
        </label>
      </div>
    </div>
  );
}
