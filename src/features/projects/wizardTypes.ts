import type { CreateProjectRequest } from '../../lib/types';

export type ProjectWizardState = CreateProjectRequest;

export const PROJECT_TYPES = [
  { value: 'vite-react', label: 'Vite React' },
  { value: 'vite-vanilla', label: 'Vite Vanilla' },
  { value: 'next', label: 'Next.js' },
  { value: 'astro', label: 'Astro' },
  { value: 'static-html', label: 'Static HTML' },
  { value: 'empty-node', label: 'Empty Node.js' },
  { value: 'php-basic', label: 'PHP Basic' },
] as const;

export const PACKAGE_MANAGERS = ['pnpm', 'npm', 'yarn'] as const;

export const initialProjectWizardState: ProjectWizardState = {
  name: '',
  path: '',
  project_type: 'vite-react',
  package_manager: 'pnpm',
  auto_install: false,
  auto_start: false,
  use_docker: false,
};
