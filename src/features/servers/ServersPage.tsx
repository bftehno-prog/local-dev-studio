import { Power } from 'lucide-react';
import type { TFunction } from '../../app/types';
import type { ServerProcess } from '../../lib/types';
import { api } from '../../shared/lib/api';
import { Panel } from '../../components/ui/Panel';
import { ServerTable } from '../../shared/ui/DataViews';

export function ServersPage({
  servers,
  onRun,
  t,
}: {
  servers: ServerProcess[];
  onRun: (action: () => Promise<unknown>, success: string) => Promise<void>;
  t: TFunction;
}) {
  return (
    <div className="content">
      <Panel title={t('servers.activeProcesses')}>
        <ServerTable servers={servers} t={t} />
        <div className="toolbar">
          <button
            onClick={() =>
              onRun(
                async () =>
                  Promise.all(servers.map((server) => api.stopProject(server.project_id))),
                t('message.allStopped'),
              )
            }
          >
            <Power size={16} /> {t('action.stopAll')}
          </button>
        </div>
      </Panel>
    </div>
  );
}
