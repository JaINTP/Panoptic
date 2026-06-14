import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { expect, test, describe, vi } from 'vitest';
import App from './App';
import { invoke } from '@tauri-apps/api/core';

describe('Panoptic React GUI Tests', () => {
  test('renders the main dashboard with default Live Overlay view', async () => {
    render(<App />);

    // Sidebar Title
    expect(screen.getByText('PANOPTIC v0.1.3')).toBeInTheDocument();

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
    const authTab = screen.getByRole('link', { name: /auth/i });
    fireEvent.click(authTab);

    // Verify Auth view is displayed
    expect(screen.getByRole('heading', { level: 1, name: 'Authentication' })).toBeInTheDocument();
    expect(screen.getByText('Developer App Settings')).toBeInTheDocument();
    expect(screen.getByText('Spotify Integration')).toBeInTheDocument();

    // Click on Output tab
    const outputTab = screen.getByRole('link', { name: /output/i });
    fireEvent.click(outputTab);

    // Verify Output view is displayed
    expect(screen.getByRole('heading', { level: 1, name: 'Output Templating' })).toBeInTheDocument();
  });

  test('submits custom client id on Auth panel', async () => {
    const alertMock = vi.spyOn(window, 'alert').mockImplementation(() => {});
    render(<App />);

    // Go to Auth tab
    fireEvent.click(screen.getByRole('link', { name: /auth/i }));

    const input = screen.getByPlaceholderText(/e.g. 299d6d15/i);
    fireEvent.change(input, { target: { value: 'my-new-client-id' } });
    expect(input).toHaveValue('my-new-client-id');

    const saveButton = screen.getByRole('button', { name: /save id/i });
    fireEvent.click(saveButton);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('set_spotify_client_id', { clientId: 'my-new-client-id' });
    });

    expect(alertMock).toHaveBeenCalledWith('Spotify Client ID saved successfully!');
    alertMock.mockRestore();
  });
});
