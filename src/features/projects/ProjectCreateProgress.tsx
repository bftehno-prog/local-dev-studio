import type { TFunction } from '../../app/types';

export function ProjectCreateProgress({
  busy,
  error,
  t,
}: {
  busy: boolean;
  error: string;
  t: TFunction;
}) {
  if (!busy && !error) {
    return null;
  }

  return (
    <div className={`wizard-status ${error ? 'error' : ''}`}>{error || t('wizard.creating')}</div>
  );
}
