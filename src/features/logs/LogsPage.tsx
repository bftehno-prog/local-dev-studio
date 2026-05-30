import type { TFunction } from '../../app/types';
import type { LogEntry } from '../../lib/types';
import { api } from '../../shared/lib/api';
import { Panel } from '../../components/ui/Panel';
import { LogList } from '../../shared/ui/DataViews';

export function LogsPage({
  logs,
  level,
  search,
  onLevel,
  onSearch,
  onRun,
  t,
}: {
  logs: LogEntry[];
  level: string;
  search: string;
  onLevel: (value: string) => void;
  onSearch: (value: string) => void;
  onRun: (action: () => Promise<unknown>, success: string) => Promise<void>;
  t: TFunction;
}) {
  return (
    <div className="content">
      <Panel title={t('logs.center')}>
        <div className="toolbar">
          <select value={level} onChange={(event) => onLevel(event.target.value)}>
            <option value="">{t('logs.all')}</option>
            <option value="info">info</option>
            <option value="warning">warning</option>
            <option value="error">error</option>
            <option value="build">build</option>
            <option value="server">server</option>
          </select>
          <input
            value={search}
            onChange={(event) => onSearch(event.target.value)}
            placeholder={t('logs.search')}
          />
          <button onClick={() => onRun(() => api.clearLogs(), t('message.logsCleared'))}>
            {t('action.clear')}
          </button>
          <button onClick={() => onRun(() => api.exportLogs(), t('message.logsExported'))}>
            {t('action.export')} .txt
          </button>
        </div>
        <LogList logs={logs} t={t} />
      </Panel>
    </div>
  );
}
