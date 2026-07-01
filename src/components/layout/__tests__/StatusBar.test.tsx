import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { StatusBar } from '../StatusBar';
import { useUIStore } from '../../../stores/uiStore';
import { useProjectStore } from '../../../stores/projectStore';
import { APP_NAME, APP_VERSION } from '../../../lib/constants';

describe('StatusBar', () => {
  beforeEach(() => {
    useUIStore.setState({
      mode: 'schematic',
      zoom: 1,
      panOffset: { x: 0, y: 0 },
      activeTool: 'select',
      sidebarTab: 'library',
      sidebarOpen: true,
      canvasApi: null,
    });
    useProjectStore.setState({
      current: null,
      loading: false,
      error: null,
    });
  });

  it('renders app name and version', () => {
    render(<StatusBar />);
    expect(screen.getByText(`${APP_NAME} v${APP_VERSION}`)).toBeInTheDocument();
  });

  it('shows "No project" when no project is open', () => {
    render(<StatusBar />);
    expect(screen.getByText('No project')).toBeInTheDocument();
  });

  it('shows project name when a project is open', () => {
    useProjectStore.setState({
      current: { id: '1', name: 'TestProject', path: '/test', updated_at: '2024-01-01' },
    });
    render(<StatusBar />);
    expect(screen.getByText('TestProject')).toBeInTheDocument();
  });

  it('shows current mode label', () => {
    useUIStore.setState({ mode: 'pcb' });
    render(<StatusBar />);
    expect(screen.getByText('PCB Layout')).toBeInTheDocument();
  });

  it('shows zoom percentage', () => {
    useUIStore.setState({ zoom: 1.5 });
    render(<StatusBar />);
    expect(screen.getByText('Zoom: 150%')).toBeInTheDocument();
  });

  it('shows 3D Preview mode', () => {
    useUIStore.setState({ mode: 'preview' });
    render(<StatusBar />);
    expect(screen.getByText('3D Preview')).toBeInTheDocument();
  });
});
