import { Database, RefreshCcw } from 'lucide-react';
import type { TFunction } from '../../app/types';
import type { DiagnosticItem, RuntimeInfo, Settings } from '../../lib/types';
import { Panel } from '../../components/ui/Panel';
import { Info, Metric } from '../../shared/ui/DataViews';

export function OnboardingPage({
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
