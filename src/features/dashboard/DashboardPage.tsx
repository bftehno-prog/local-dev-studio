import type { TFunction } from '../../app/types';
import type { DashboardData, PortInfo, Project, ServerProcess } from '../../lib/types';
import { Panel } from '../../components/ui/Panel';
import {
  Info,
  LogList,
  Metric,
  PortList,
  runtimeText,
  ServerTable,
  versionText,
} from '../../shared/ui/DataViews';

export function DashboardPage({
  dashboard,
  servers,
  ports,
  projects,
  t,
}: {
  dashboard: DashboardData | null;
  servers: ServerProcess[];
  ports: PortInfo[];
  projects: Project[];
  t: TFunction;
}) {
  return (
    <div className="content">
      <div className="metrics">
        <Metric label={t('dashboard.running')} value={dashboard?.running_projects ?? 0} />
        <Metric
          label={t('dashboard.stopped')}
          value={dashboard?.stopped_projects ?? projects.length}
        />
        <Metric
          label={t('dashboard.usedPorts')}
          value={dashboard?.used_ports.join(', ') || t('empty.none')}
        />
        <Metric label={t('dashboard.runtime')} value={runtimeText(dashboard?.runtime_status, t)} />
      </div>
      <div className="grid two">
        <Panel title={t('dashboard.environment')}>
          <Info label={t('env.node')} value={versionText(dashboard?.node_version, t)} />
          <Info label={t('env.npm')} value={versionText(dashboard?.npm_version, t)} />
          <Info label={t('env.pnpm')} value={versionText(dashboard?.pnpm_version, t)} />
          <Info label={t('env.git')} value={versionText(dashboard?.git_version, t)} />
          <Info label={t('env.php')} value={versionText(dashboard?.php_version, t)} />
        </Panel>
        <Panel title={t('dashboard.activeServers')}>
          <ServerTable servers={servers} compact t={t} />
        </Panel>
      </div>
      <div className="grid two">
        <Panel title={t('dashboard.recentErrors')}>
          <LogList logs={dashboard?.recent_errors ?? []} t={t} />
        </Panel>
        <Panel title={t('dashboard.ports')}>
          <PortList ports={ports.filter((port) => !port.available).slice(0, 12)} t={t} />
        </Panel>
      </div>
    </div>
  );
}
