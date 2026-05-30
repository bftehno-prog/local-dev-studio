import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ProjectWizard } from './ProjectWizard';

const t = (key: string) => key;

describe('ProjectWizard', () => {
  it('renders project creation controls', () => {
    render(<ProjectWizard onCreated={() => undefined} t={t} />);

    expect(screen.getByRole('heading', { name: 'wizard.title' })).toBeInTheDocument();
    expect(screen.getByText('wizard.projectType')).toBeInTheDocument();
    expect(screen.getByText('wizard.projectName')).toBeInTheDocument();
    expect(screen.getByText('wizard.projectPath')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /action.create/ })).toBeDisabled();
  });
});
