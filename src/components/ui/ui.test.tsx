import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { EmptyState } from './EmptyState';
import { Panel } from './Panel';
import { StatusBadge } from './StatusBadge';

describe('ui primitives', () => {
  it('renders status badge', () => {
    render(<StatusBadge status="running" />);
    expect(screen.getByText('running')).toHaveClass('status', 'running');
  });

  it('renders empty state with action', () => {
    render(<EmptyState action={<button>Start</button>}>No projects</EmptyState>);
    expect(screen.getByText('No projects')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Start' })).toBeInTheDocument();
  });

  it('renders panel title and content', () => {
    render(
      <Panel title="Settings">
        <span>Runtime</span>
      </Panel>,
    );
    expect(screen.getByRole('heading', { name: 'Settings' })).toBeInTheDocument();
    expect(screen.getByText('Runtime')).toBeInTheDocument();
  });
});
