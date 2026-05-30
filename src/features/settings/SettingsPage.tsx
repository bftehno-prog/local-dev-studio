import { useState } from 'react';
import { RefreshCcw } from 'lucide-react';
import type { TFunction } from '../../app/types';
import type { Language } from '../../lib/i18n';
import type { DiagnosticItem, RuntimeInfo, Settings } from '../../lib/types';
import { Panel } from '../../components/ui/Panel';

export function SettingsPage({
  settings,
  onChange,
  onSave,
  diagnostics,
  runtimes,
  onRefreshDiagnostics,
  onRefreshRuntimes,
  t,
  language,
}: {
  settings: Settings;
  onChange: (settings: Settings) => void;
  onSave: () => Promise<void>;
  diagnostics: DiagnosticItem[];
  runtimes: RuntimeInfo[];
  onRefreshDiagnostics: () => Promise<void>;
  onRefreshRuntimes: () => Promise<void>;
  t: TFunction;
  language: Language;
}) {
  const [settingsTab, setSettingsTab] = useState('general');
  const set = <K extends keyof Settings>(key: K, value: Settings[K]) =>
    onChange({ ...settings, [key]: value });
  const tabs = [
    ['general', t('settings.general')],
    ['runtime', t('settings.runtime')],
    ['servers', t('settings.servers')],
    ['preview', t('settings.preview')],
    ['next', t('settings.next')],
    ['experimental', t('settings.experimental')],
    ['diagnostics', t('settings.diagnostics')],
  ];

  return (
    <div className="content">
      <div className="settings-tabs" role="tablist" aria-label={t('nav.settings')}>
        {tabs.map(([id, label]) => (
          <button
            key={id}
            className={settingsTab === id ? 'active' : ''}
            role="tab"
            aria-selected={settingsTab === id}
            onClick={() => setSettingsTab(id)}
          >
            {label}
          </button>
        ))}
      </div>

      {settingsTab === 'general' && (
        <Panel title={t('settings.general')}>
          <label className="field">
            <span>{t('settings.language')}</span>
            <select
              value={language}
              onChange={(event) => set('language', event.target.value as Language)}
            >
              <option value="en">English</option>
              <option value="ru">Русский</option>
            </select>
          </label>
          <Field
            label={t('settings.projectsFolder')}
            value={settings.projects_folder}
            onChange={(value) => set('projects_folder', value)}
          />
          <Field
            label={t('settings.sandboxesFolder')}
            value={settings.sandboxes_folder}
            onChange={(value) => set('sandboxes_folder', value)}
          />
          <Toggle
            label={t('settings.openPreview')}
            checked={settings.open_preview_automatically}
            onChange={(value) => set('open_preview_automatically', value)}
            disabled
          />
          <Toggle
            label={t('settings.startMinimized')}
            checked={settings.start_minimized}
            onChange={(value) => set('start_minimized', value)}
          />
          <Toggle
            label={t('settings.launchOnStartup')}
            checked={settings.launch_on_startup}
            onChange={(value) => set('launch_on_startup', value)}
          />
        </Panel>
      )}

      {settingsTab === 'runtime' && (
        <Panel title={t('settings.runtime')}>
          <div className="toolbar">
            <button onClick={() => void onRefreshRuntimes()}>
              <RefreshCcw size={16} /> {t('action.rerunDiagnostics')}
            </button>
          </div>
          <Toggle
            label={t('settings.useBundledNode')}
            checked={settings.use_bundled_node}
            onChange={(value) => set('use_bundled_node', value)}
          />
          <Field
            label={t('settings.packageManager')}
            value={settings.package_manager}
            onChange={(value) => set('package_manager', value)}
          />
          <Field
            label={t('settings.nodePath')}
            value={settings.node_path}
            onChange={(value) => set('node_path', value)}
          />
          <Field
            label={t('settings.npmPath')}
            value={settings.npm_path}
            onChange={(value) => set('npm_path', value)}
          />
          <Field
            label={t('settings.pnpmPath')}
            value={settings.pnpm_path}
            onChange={(value) => set('pnpm_path', value)}
          />
          <Field
            label={t('settings.yarnPath')}
            value={settings.yarn_path}
            onChange={(value) => set('yarn_path', value)}
          />
          <Field
            label={t('settings.bunPath')}
            value={settings.bun_path}
            onChange={(value) => set('bun_path', value)}
          />
          <Field
            label={t('settings.phpPath')}
            value={settings.php_path}
            onChange={(value) => set('php_path', value)}
          />
          <Field
            label={t('settings.gitPath')}
            value={settings.git_path}
            onChange={(value) => set('git_path', value)}
          />
          <h2>{t('settings.runtimeHealth')}</h2>
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
      )}

      {settingsTab === 'servers' && (
        <Panel title={t('settings.servers')}>
          <NumberField
            label={t('settings.portStart')}
            value={settings.port_start}
            onChange={(value) => set('port_start', value)}
          />
          <NumberField
            label={t('settings.portEnd')}
            value={settings.port_end}
            onChange={(value) => set('port_end', value)}
          />
          <NumberField
            label={t('settings.processTimeout')}
            value={settings.process_timeout}
            onChange={(value) => set('process_timeout', value)}
          />
          <NumberField
            label={t('settings.logRetention')}
            value={settings.log_retention}
            onChange={(value) => set('log_retention', value)}
          />
          <Field
            label={t('settings.envVars')}
            value={settings.environment_variables}
            onChange={(value) => set('environment_variables', value)}
          />
        </Panel>
      )}

      {settingsTab === 'preview' && (
        <Panel title={t('settings.preview')}>
          <Field
            label={t('settings.defaultDevice')}
            value={settings.default_device}
            onChange={(value) => set('default_device', value)}
          />
          <NumberField
            label={t('settings.customWidth')}
            value={settings.custom_width}
            onChange={(value) => set('custom_width', value)}
          />
          <Toggle
            label={t('settings.networkPreview')}
            checked={settings.enable_network_preview}
            onChange={(value) => set('enable_network_preview', value)}
          />
          <Toggle
            label={t('settings.autoReloadPreview')}
            checked={settings.auto_reload_preview}
            onChange={(value) => set('auto_reload_preview', value)}
          />
          <Toggle
            label={t('settings.openExternalBrowser')}
            checked={settings.open_external_browser_on_start}
            onChange={(value) => set('open_external_browser_on_start', value)}
          />
        </Panel>
      )}

      {settingsTab === 'next' && (
        <Panel title={t('settings.next')}>
          <Toggle
            label={t('settings.useTurbopack')}
            checked={settings.use_turbopack}
            onChange={(value) => set('use_turbopack', value)}
          />
          <Toggle
            label={t('settings.clearNext')}
            checked={settings.clear_next_before_start}
            onChange={(value) => set('clear_next_before_start', value)}
          />
          <NumberField
            label={t('settings.defaultNextPort')}
            value={settings.default_next_port}
            onChange={(value) => set('default_next_port', value)}
          />
        </Panel>
      )}

      {settingsTab === 'experimental' && (
        <Panel title={t('settings.experimental')}>
          <Toggle
            label={`${t('settings.https')} (${t('status.inDevelopment')})`}
            checked={settings.enable_https}
            onChange={(value) => set('enable_https', value)}
            disabled
          />
          <Field
            label={`${t('settings.proxyRules')} (${t('status.inDevelopment')})`}
            value={settings.proxy_rules}
            onChange={(value) => set('proxy_rules', value)}
            disabled
          />
          <Field
            label={`${t('settings.hosts')} (${t('status.inDevelopment')})`}
            value={settings.hosts}
            onChange={(value) => set('hosts', value)}
            disabled
          />
          <Field
            label={`${t('settings.sslCertificates')} (${t('status.inDevelopment')})`}
            value={settings.ssl_certificates}
            onChange={(value) => set('ssl_certificates', value)}
            disabled
          />
        </Panel>
      )}

      {settingsTab === 'diagnostics' && (
        <Panel title={t('settings.diagnostics')}>
          <div className="toolbar">
            <button onClick={() => void onRefreshDiagnostics()}>
              <RefreshCcw size={16} /> {t('action.rerunDiagnostics')}
            </button>
          </div>
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
      )}
      <button className="primary-save" onClick={() => void onSave()}>
        {t('action.saveSettings')}
      </button>
    </div>
  );
}

function Field({
  label,
  value,
  onChange,
  disabled = false,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
}) {
  return (
    <label className="field">
      <span>{label}</span>
      <input value={value} disabled={disabled} onChange={(event) => onChange(event.target.value)} />
    </label>
  );
}

function NumberField({
  label,
  value,
  onChange,
}: {
  label: string;
  value: number;
  onChange: (value: number) => void;
}) {
  return (
    <label className="field">
      <span>{label}</span>
      <input
        type="number"
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
      />
    </label>
  );
}

function Toggle({
  label,
  checked,
  onChange,
  disabled = false,
}: {
  label: string;
  checked: boolean;
  onChange: (value: boolean) => void;
  disabled?: boolean;
}) {
  return (
    <label className="toggle">
      <input
        type="checkbox"
        checked={checked}
        disabled={disabled}
        onChange={(event) => onChange(event.target.checked)}
      />{' '}
      <span>{label}</span>
    </label>
  );
}
