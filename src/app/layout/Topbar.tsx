import { Play, Plus, Power, RefreshCcw, Upload } from 'lucide-react';
import { sections, type SectionId } from '../routes';
import type { TFunction } from '../types';

export function Topbar({
  active,
  message,
  onRefresh,
  onStartAll,
  onStopAll,
  onAddProject,
  onImportZip,
  t,
}: {
  active: SectionId;
  message: string;
  onRefresh: () => void;
  onStartAll: () => void;
  onStopAll: () => void;
  onAddProject: () => void;
  onImportZip: () => void;
  t: TFunction;
}) {
  return (
    <header className="topbar">
      <div>
        <h1>{t(sections.find(([id]) => id === active)?.[1] ?? 'nav.dashboard')}</h1>
        <p>{message || t('top.subtitle')}</p>
      </div>
      <div className="top-actions">
        <button onClick={onRefresh}>
          <RefreshCcw size={16} /> {t('action.refresh')}
        </button>
        <button onClick={onStartAll}>
          <Play size={16} /> {t('action.startAll')}
        </button>
        <button onClick={onStopAll}>
          <Power size={16} /> {t('action.stopAll')}
        </button>
        <button onClick={onAddProject}>
          <Plus size={16} /> {t('action.addProject')}
        </button>
        <button onClick={onImportZip}>
          <Upload size={16} /> {t('action.importZip')}
        </button>
      </div>
    </header>
  );
}
