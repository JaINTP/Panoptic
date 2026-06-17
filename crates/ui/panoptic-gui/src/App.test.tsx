import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { expect, test, describe } from 'vitest';
import App from './App';
import { invoke } from '@tauri-apps/api/core';

describe('Panoptic React GUI Tests', () => {
  test('renders the main dashboard with default Live Overlay view', async () => {
    render(<App />);

    // Sidebar Title
    expect(screen.getByText('PANOPTIC v0.1.6')).toBeInTheDocument();

    // Default view content (Live Overlay Preview)
    expect(screen.getByText('Live Overlay Preview')).toBeInTheDocument();

    // Mock song details from the initial state
    expect(screen.getByText('Resonance')).toBeInTheDocument();
    expect(screen.getByText('Home')).toBeInTheDocument();
    expect(screen.getByText('Odyssey')).toBeInTheDocument();

    // Check time rendering (165000ms -> 2:45, 210000ms -> 3:30)
    expect(screen.getByText('2:45')).toBeInTheDocument();
    expect(screen.getByText('3:30')).toBeInTheDocument();
  });

  test('switches views when clicking sidebar items', async () => {
    render(<App />);

    // Click on Auth tab
    const authTab = screen.getByRole('button', { name: /auth/i });
    fireEvent.click(authTab);

    // Verify Auth view is displayed
    expect(screen.getByRole('heading', { level: 1, name: 'Authentication' })).toBeInTheDocument();
    
    // Check for dynamic plugin content
    await waitFor(() => {
      expect(screen.getByText('Spotify')).toBeInTheDocument();
      expect(screen.getByText('Custom Client ID')).toBeInTheDocument();
    });

    // Click on Output tab
    const outputTab = screen.getByRole('button', { name: /output/i });
    fireEvent.click(outputTab);

    // Verify Output view is displayed
    expect(screen.getByRole('heading', { level: 1, name: 'Output Templating' })).toBeInTheDocument();
  });

  test('updates plugin settings on Auth panel', async () => {
    render(<App />);

    // Go to Auth tab
    fireEvent.click(screen.getByRole('button', { name: /auth/i }));

    await waitFor(() => {
      expect(screen.getByText('Spotify')).toBeInTheDocument();
    });

    const input = screen.getByDisplayValue('mock-client-id-123');
    fireEvent.change(input, { target: { value: 'my-new-client-id' } });
    
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('set_plugin_settings', expect.objectContaining({
        pluginId: 'spotify',
        newSettings: expect.objectContaining({
          client_id: 'my-new-client-id'
        })
      }));
    });
  });
});
