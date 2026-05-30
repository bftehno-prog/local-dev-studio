import { Database } from 'lucide-react';
import { sections, type SectionId } from '../routes';
import type { TFunction } from '../types';

export function Sidebar({
  active,
  onSelect,
  t,
}: {
  active: SectionId;
  onSelect: (section: SectionId) => void;
  t: TFunction;
}) {
  return (
    <aside className="sidebar">
      <div className="brand">
        <Database size={22} />
        <div>
          <strong>Local Dev Studio</strong>
          <span>Windows</span>
        </div>
      </div>
      <nav>
        {sections.map(([id, labelKey, Icon]) => (
          <button
            className={active === id ? 'nav-item active' : 'nav-item'}
            key={id}
            onClick={() => onSelect(id)}
          >
            <Icon size={17} />
            {t(labelKey)}
          </button>
        ))}
      </nav>
    </aside>
  );
}
