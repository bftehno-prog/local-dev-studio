import { Play } from 'lucide-react';
import type { TFunction } from '../../app/types';
import type { TemplateInfo } from '../../lib/types';
import { api } from '../../shared/lib/api';
import { Panel } from '../../components/ui/Panel';
import { templateName } from '../../shared/ui/DataViews';

export function SandboxesPage({
  templates,
  onRun,
  t,
}: {
  templates: TemplateInfo[];
  onRun: (action: () => Promise<unknown>, success: string) => Promise<void>;
  t: TFunction;
}) {
  return (
    <div className="content">
      <Panel title={t('sandbox.create')}>
        <div className="template-grid">
          {templates
            .filter((template) => template.built_in)
            .map((template) => (
              <article className="template-card" key={template.id}>
                <strong>{templateName(template, t)}</strong>
                <span>{template.project_type}</span>
                <button
                  onClick={() =>
                    onRun(() => api.createSandbox(template.id), t('message.sandboxCreated'))
                  }
                >
                  <Play size={16} /> {t('action.create')}
                </button>
              </article>
            ))}
        </div>
      </Panel>
    </div>
  );
}
