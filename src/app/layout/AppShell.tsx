import type { ReactNode } from 'react';
import type { SectionId } from '../routes';
import type { TFunction } from '../types';
import { Sidebar } from './Sidebar';
import { Topbar } from './Topbar';

export function AppShell({
  active,
  message,
  onSelectSection,
  onRefresh,
  onStartAll,
  onStopAll,
  onAddProject,
  onImportZip,
  preview,
  children,
  t,
}: {
  active: SectionId;
  message: string;
  onSelectSection: (section: SectionId) => void;
  onRefresh: () => void;
  onStartAll: () => void;
  onStopAll: () => void;
  onAddProject: () => void;
  onImportZip: () => void;
  preview: ReactNode;
  children: ReactNode;
  t: TFunction;
}) {
  return (
    <main className="app">
      <Sidebar active={active} onSelect={onSelectSection} t={t} />
      <section className="workspace">
        <Topbar
          active={active}
          message={message}
          onRefresh={onRefresh}
          onStartAll={onStartAll}
          onStopAll={onStopAll}
          onAddProject={onAddProject}
          onImportZip={onImportZip}
          t={t}
        />
        {children}
      </section>
      {preview}
    </main>
  );
}
