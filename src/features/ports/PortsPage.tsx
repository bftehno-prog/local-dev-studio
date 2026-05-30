import type { TFunction } from '../../app/types';
import type { PortInfo } from '../../lib/types';
import { api } from '../../shared/lib/api';
import { Panel } from '../../components/ui/Panel';

export function PortsPage({
  ports,
  onRun,
  t,
}: {
  ports: PortInfo[];
  onRun: (action: () => Promise<unknown>, success: string) => Promise<void>;
  t: TFunction;
}) {
  return (
    <div className="content">
      <Panel title={t('ports.manager')}>
        <table>
          <thead>
            <tr>
              <th>{t('table.port')}</th>
              <th>{t('table.status')}</th>
              <th>{t('table.project')}</th>
              <th>{t('table.pid')}</th>
              <th>{t('table.actions')}</th>
            </tr>
          </thead>
          <tbody>
            {ports.map((port) => (
              <tr key={port.port}>
                <td>{port.port}</td>
                <td>
                  {port.available
                    ? t('ports.free')
                    : port.external
                      ? t('ports.external')
                      : t('ports.managed')}
                </td>
                <td>{port.project_name || '-'}</td>
                <td>{port.pid || '-'}</td>
                <td>
                  {!port.available && (
                    <button
                      onClick={() =>
                        onRun(() => api.releasePort(port.port), t('message.portReleased'))
                      }
                    >
                      {t('action.release')}
                    </button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </Panel>
    </div>
  );
}
