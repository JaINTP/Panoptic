import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Mock Tauri core APIs
vi.mock('@tauri-apps/api/core', () => {
  return {
    invoke: vi.fn(async (cmd, args: any) => {
      console.log(`Mock invoke called for: ${cmd}`, args);
      if (cmd === 'get_plugins_metadata') {
        return [{
          id: 'spotify',
          name: 'Spotify',
          category: 'auth',
          fields: [
            {
              key: 'client_id',
              label: 'Custom Client ID',
              description: 'Register your own app.',
              field_type: { type: 'Text' },
              default_value: ''
            },
            {
              key: 'link_action',
              label: 'Spotify Integration',
              field_type: { 
                type: 'Action', 
                options: { button_label: 'Link Spotify', action_name: 'link' } 
              },
              default_value: null
            }
          ]
        }];
      }
      if (cmd === 'get_plugin_settings') {
        if (args?.pluginId === 'spotify') {
          return { client_id: 'mock-client-id-123' };
        }
        return {};
      }
      if (cmd === 'get_not_playing_settings') {
        return {
          not_playing_title: 'Not Playing',
          not_playing_artist: 'Unknown Artist',
          not_playing_album: 'Unknown Album'
        };
      }
      if (cmd === 'get_overlay_css') {
        return '/* Mock CSS */';
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
