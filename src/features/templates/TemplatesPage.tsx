import { useEffect, useState } from 'react';
import { Folder, Upload } from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import type { TFunction } from '../../app/types';
import type { TemplateInfo } from '../../lib/types';
import { api } from '../../shared/lib/api';
import { Panel } from '../../components/ui/Panel';
import { templateName } from '../../shared/ui/DataViews';

export function TemplatesPage({
  templates,
  onRun,
  t,
}: {
  templates: TemplateInfo[];
  onRun: (action: () => Promise<unknown>, success: string) => Promise<void>;
  t: TFunction;
}) {
  const [zipPath, setZipPath] = useState('');
  const [zipMessage, setZipMessage] = useState('');

  function acceptZip(path: string) {
    if (!path.toLowerCase().endsWith('.zip')) {
      setZipMessage(t('templates.zipOnly'));
      return;
    }
    setZipPath(path);
    setZipMessage('');
    window.localStorage.setItem('local-dev-studio:lastZipPath', path);
  }

  async function chooseZip() {
    const selected = await open({
      directory: false,
      multiple: false,
      title: t('templates.selectZip'),
      filters: [{ name: 'ZIP', extensions: ['zip'] }],
    });
    if (typeof selected === 'string') {
      acceptZip(selected);
    }
  }

  useEffect(() => {
    setZipPath(window.localStorage.getItem('local-dev-studio:lastZipPath') || '');
  }, []);

  return (
    <div className="content">
      <Panel title={t('templates.title')}>
        <div
          className="drop-zone"
          onDragOver={(event) => event.preventDefault()}
          onDrop={(event) => {
            event.preventDefault();
            const files = Array.from(event.dataTransfer.files);
            const zip = files.find((file) => file.name.toLowerCase().endsWith('.zip'));
            if (!zip) {
              setZipMessage(t('templates.dropZipWarning'));
              return;
            }
            acceptZip((zip as File & { path?: string }).path || zip.name);
          }}
        >
          <Upload size={18} />
          <span>{t('templates.dropZip')}</span>
        </div>
        <div className="toolbar">
          <input
            value={zipPath}
            onChange={(event) => setZipPath(event.target.value)}
            placeholder="C:\Users\User\Downloads\template.zip"
          />
          <button onClick={() => void chooseZip()}>
            <Folder size={16} /> {t('action.chooseZip')}
          </button>
          <button
            onClick={() =>
              onRun(() => api.importTemplateZip(zipPath), t('message.templateImported'))
            }
          >
            {t('action.importZip')}
          </button>
        </div>
        {zipMessage && <p className="muted">{zipMessage}</p>}
        <table>
          <thead>
            <tr>
              <th>{t('table.name')}</th>
              <th>{t('table.type')}</th>
              <th>{t('table.source')}</th>
              <th>{t('table.actions')}</th>
            </tr>
          </thead>
          <tbody>
            {templates.map((template) => (
              <tr key={template.id}>
                <td>{templateName(template, t)}</td>
                <td>{template.project_type}</td>
                <td>{template.built_in ? t('templates.builtin') : t('templates.user')}</td>
                <td className="actions">
                  <button
                    onClick={() =>
                      onRun(() => api.createFromTemplate(template.id), t('message.templateCreated'))
                    }
                  >
                    {t('action.create')}
                  </button>
                  <button
                    onClick={() =>
                      onRun(
                        () => api.duplicateTemplate(template.id),
                        t('message.templateDuplicated'),
                      )
                    }
                  >
                    {t('action.duplicate')}
                  </button>
                  <button
                    onClick={() =>
                      onRun(() => api.exportTemplateZip(template.id), t('message.templateExported'))
                    }
                  >
                    {t('action.export')}
                  </button>
                  {!template.built_in && (
                    <button
                      onClick={() =>
                        onRun(() => api.deleteTemplate(template.id), t('message.templateDeleted'))
                      }
                    >
                      {t('action.delete')}
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
