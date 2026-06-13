import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Mock Tauri core APIs
vi.mock('@tauri-apps/api/core', () => {
  return {
    invoke: vi.fn(async (cmd, args) => {
      console.log(`Mock invoke called for: ${cmd}`, args);
      if (cmd === 'get_spotify_client_id') {
        return 'mock-client-id-123';
      }
      if (cmd === 'get_spotify_status') {
        return true;
      }
      if (cmd === 'get_output_template') {
        return 'Now Playing: {title} by {artist}';
      }
      return null;
    }),
  };
});

// Mock Tauri event APIs
vi.mock('@tauri-apps/api/event', () => {
  return {
    listen: vi.fn(async (eventName, _callback) => {
      console.log(`Mock listen registered for event: ${eventName}`);
      // Return unlisten function
      return () => {
        console.log(`Mock unlisten called for: ${eventName}`);
      };
    }),
  };
});
