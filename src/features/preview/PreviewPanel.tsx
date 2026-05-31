import { Copy, Globe2, Link, Play, RefreshCcw, Square } from 'lucide-react';
import { QRCodeSVG } from 'qrcode.react';
import { devices } from '../../app/routes';
import type { TFunction } from '../../app/types';
import { api } from '../../shared/api/commands';
import type { Project, ProxyStatus, ServerProcess, Settings } from '../../lib/types';
import type { TranslationKey } from '../../lib/i18n';

type PreviewPanelProps = {
  t: TFunction;
  settings: Settings;
  servers: ServerProcess[];
  selectedProject?: Project;
  activePreviewServerId: string;
  setActivePreviewServerId: (value: string) => void;
  previewUrl: string;
  setPreviewUrl: (value: string) => void;
  manualPreviewUrl: string;
  setManualPreviewUrl: (value: string) => void;
  previewKey: number;
  setPreviewKey: React.Dispatch<React.SetStateAction<number>>;
  fitPreview: boolean;
  setFitPreview: React.Dispatch<React.SetStateAction<boolean>>;
  previewLoading: boolean;
  setPreviewLoading: (value: boolean) => void;
  previewError: string;
  setPreviewError: (value: string) => void;
  device: string;
  setDevice: (value: string) => void;
  previewWidth: number;
  previewScale: number;
  localPreviewUrl: string;
  networkPreviewUrl: string;
  activePreviewServer?: ServerProcess;
  proxyStatus: ProxyStatus | null;
  setProxyStatus: (value: ProxyStatus | null) => void;
  showError: (error: unknown) => void;
  run: (action: () => Promise<unknown>, success: string) => Promise<void>;
};

export function PreviewPanel({
  t,
  servers,
  selectedProject,
  activePreviewServerId,
  setActivePreviewServerId,
  previewUrl,
  setPreviewUrl,
  manualPreviewUrl,
  setManualPreviewUrl,
  previewKey,
  setPreviewKey,
  fitPreview,
  setFitPreview,
  previewLoading,
  setPreviewLoading,
  previewError,
  setPreviewError,
  device,
  setDevice,
  previewWidth,
  previewScale,
  localPreviewUrl,
  networkPreviewUrl,
  activePreviewServer,
  proxyStatus,
  setProxyStatus,
  showError,
  run,
}: PreviewPanelProps) {
  const proxyProjectId = activePreviewServer?.project_id || selectedProject?.id || '';

  async function activateProxyPreview(action: 'start' | 'restart') {
    if (!proxyProjectId) return;
    await run(
      async () => {
        const status =
          action === 'start'
            ? await api.startProxy(proxyProjectId)
            : await api.restartProxy(proxyProjectId);
        setProxyStatus(status);
        if (status.preview_url) {
          setManualPreviewUrl('');
          setPreviewUrl(status.preview_url);
          setActivePreviewServerId(status.project_id);
          setPreviewKey((value) => value + 1);
        }
      },
      action === 'start' ? t('message.proxyStarted') : t('message.proxyRestarted'),
    );
  }

  async function stopProxyPreview() {
    if (!proxyProjectId) return;
    await run(async () => {
      const status = await api.stopProxy(proxyProjectId);
      setProxyStatus(status);
    }, t('message.proxyStopped'));
  }

  return (
    <aside className="preview">
      <div className="preview-header">
        <div>
          <strong>{t('preview.title')}</strong>
          <span>{localPreviewUrl || t('preview.noServer')}</span>
        </div>
        <div className="preview-actions">
          <button
            disabled={!localPreviewUrl}
            title={t('preview.reload')}
            onClick={() => setPreviewKey((value) => value + 1)}
          >
            <RefreshCcw size={16} />
          </button>
          <button
            disabled={!localPreviewUrl}
            title={t('preview.copyUrl')}
            onClick={() =>
              localPreviewUrl && navigator.clipboard.writeText(localPreviewUrl).catch(showError)
            }
          >
            <Copy size={16} />
          </button>
          <button
            disabled={!localPreviewUrl}
            title={t('preview.openBrowser')}
            onClick={() =>
              localPreviewUrl && void api.openExternal(localPreviewUrl).catch(showError)
            }
          >
            <Globe2 size={16} />
          </button>
        </div>
      </div>
      <div className="preview-controls">
        <select
          value={activePreviewServerId}
          onChange={(event) => {
            const server = servers.find((item) => item.project_id === event.target.value);
            setActivePreviewServerId(event.target.value);
            setManualPreviewUrl('');
            setPreviewUrl(server?.url ?? '');
            setPreviewKey((value) => value + 1);
          }}
        >
          <option value="">{t('preview.selectServer')}</option>
          {servers.map((server) => (
            <option key={server.project_id} value={server.project_id}>
              {server.project_name} :{server.port}
            </option>
          ))}
        </select>
        <div className="url-row">
          <Link size={15} />
          <input
            value={manualPreviewUrl || previewUrl}
            onChange={(event) => {
              setManualPreviewUrl(event.target.value);
              setPreviewUrl(event.target.value);
            }}
            placeholder="http://localhost:3000"
          />
        </div>
        <div className="proxy-row">
          <span className={`status ${proxyStatus?.running ? 'running' : ''}`}>
            {t('preview.proxy')}: {proxyStatus?.running ? t('status.running') : t('status.stopped')}
          </span>
          <button disabled={!proxyProjectId} onClick={() => void activateProxyPreview('start')}>
            <Link size={15} /> {t('action.startProxy')}
          </button>
          <button
            disabled={!proxyProjectId || !proxyStatus?.running}
            onClick={() => void activateProxyPreview('restart')}
          >
            <RefreshCcw size={15} /> {t('action.restartProxy')}
          </button>
          <button
            disabled={!proxyProjectId || !proxyStatus?.running}
            onClick={() => void stopProxyPreview()}
          >
            <Square size={15} /> {t('action.stopProxy')}
          </button>
        </div>
      </div>
      <div className="device-tabs">
        {devices.map(([name]) => (
          <button
            key={name}
            className={device === name ? 'active' : ''}
            onClick={() => setDevice(name)}
          >
            {t(`device.${name}` as TranslationKey)}
          </button>
        ))}
        <button
          className={fitPreview ? 'active' : ''}
          onClick={() => setFitPreview((value) => !value)}
        >
          {fitPreview ? t('preview.fit') : t('preview.actual')}
        </button>
      </div>
      <div className="preview-frame-shell">
        {localPreviewUrl ? (
          <div
            className="preview-frame-scale"
            style={{ width: previewWidth, transform: `scale(${previewScale})` }}
          >
            {previewLoading && <div className="preview-state">{t('preview.loading')}</div>}
            {previewError && <div className="preview-state error">{previewError}</div>}
            <iframe
              key={`${localPreviewUrl}-${device}-${previewKey}`}
              title={t('preview.iframeTitle')}
              src={localPreviewUrl}
              onLoad={() => {
                setPreviewLoading(false);
                setPreviewError('');
              }}
              onError={() => {
                setPreviewLoading(false);
                setPreviewError(t('preview.unavailable'));
              }}
            />
          </div>
        ) : (
          <div className="empty-preview">
            <p>{t('preview.placeholder')}</p>
            {selectedProject && (
              <button
                onClick={() =>
                  void run(() => api.startProject(selectedProject.id), t('message.projectStarted'))
                }
              >
                <Play size={16} /> {t('action.start')}
              </button>
            )}
          </div>
        )}
      </div>
      {localPreviewUrl && (
        <div className="qr-row">
          <QRCodeSVG value={networkPreviewUrl} size={88} />
          <div>
            <span>{t('preview.local')}</span>
            <code>{localPreviewUrl}</code>
            <span>{t('preview.network')}</span>
            <code>{networkPreviewUrl}</code>
            {activePreviewServer && (
              <>
                <span>{t('preview.health')}</span>
                <code>{activePreviewServer.status}</code>
              </>
            )}
            {proxyStatus?.preview_url && (
              <>
                <span>{t('preview.proxyUrl')}</span>
                <code>{proxyStatus.preview_url}</code>
              </>
            )}
          </div>
        </div>
      )}
    </aside>
  );
}
