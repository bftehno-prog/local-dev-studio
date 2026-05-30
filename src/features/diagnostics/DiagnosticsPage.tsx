import { RefreshCcw } from 'lucide-react';
import type { TFunction } from '../../app/types';
import type { DiagnosticItem, RuntimeInfo } from '../../lib/types';
import { Panel } from '../../components/ui/Panel';
import { Metric } from '../../shared/ui/DataViews';

export function DiagnosticsPage({
  diagnostics,
  runtimes,
  onRefresh,
  onCopyReport,
  onOpenLogs,
  t,
}: {
  diagnostics: DiagnosticItem[];
  runtimes: RuntimeInfo[];
  onRefresh: () => void;
  onCopyReport: () => void;
  onOpenLogs: () => void;
  t: TFunction;
}) {
  return (
    <div className="content">
      <Panel title={t('nav.diagnostics')}>
        <div className="toolbar">
          <button onClick={onRefresh}>
            <RefreshCcw size={16} /> {t('action.rerunDiagnostics')}
          </button>
          <button onClick={onCopyReport}>{t('action.copyReport')}</button>
          <button onClick={onOpenLogs}>{t('nav.logs')}</button>
        </div>
        <div className="metrics">
          <Metric
            label={t('settings.runtimeHealth')}
            value={`${runtimes.filter((runtime) => runtime.found).length}/${runtimes.length}`}
          />
          <Metric
            label={t('table.error')}
            value={
              diagnostics.filter((item) => item.status === 'Error' || item.status === 'Missing')
                .length
            }
          />
          <Metric
            label="SQLite"
            value={diagnostics.find((item) => item.name === 'SQLite data')?.status ?? '-'}
          />
          <Metric
            label="PATH"
            value={diagnostics.find((item) => item.name === 'PATH')?.status ?? '-'}
          />
        </div>
      </Panel>
      <Panel title={t('settings.runtimeHealth')}>
        <table>
          <thead>
            <tr>
              <th>{t('table.runtime')}</th>
              <th>{t('table.status')}</th>
              <th>{t('table.version')}</th>
              <th>{t('settings.source')}</th>
              <th>{t('table.path')}</th>
              <th>{t('table.error')}</th>
            </tr>
          </thead>
          <tbody>
            {runtimes.map((runtime) => (
              <tr key={runtime.name}>
                <td>{runtime.name}</td>
                <td>
                  <span className={`status ${runtime.found ? 'running' : 'error'}`}>
                    {runtime.found ? 'OK' : t('empty.notFound')}
                  </span>
                </td>
                <td>
                  <code>{runtime.version || '-'}</code>
                </td>
                <td>{runtime.source}</td>
                <td>
                  <code>{runtime.path || '-'}</code>
                </td>
                <td>{runtime.error || '-'}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </Panel>
      <Panel title={t('settings.diagnostics')}>
        <table>
          <thead>
            <tr>
              <th>{t('table.name')}</th>
              <th>{t('table.status')}</th>
              <th>{t('table.version')}</th>
              <th>{t('table.path')}</th>
              <th>{t('table.error')}</th>
            </tr>
          </thead>
          <tbody>
            {diagnostics.map((item) => (
              <tr key={`${item.name}-${item.path}`}>
                <td>{item.name}</td>
                <td>
                  <span className={`status ${item.status.toLowerCase()}`}>{item.status}</span>
                </td>
                <td>
                  <code>{item.version || '-'}</code>
                </td>
                <td>
                  <code>{item.path || '-'}</code>
                </td>
                <td>{item.error || '-'}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </Panel>
    </div>
  );
}
