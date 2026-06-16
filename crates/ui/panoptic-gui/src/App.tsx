import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { openUrl } from '@tauri-apps/plugin-opener';

import { Sidebar, View } from './components/Sidebar';
import { PlaybackState } from './components/OverlayPreview';
import { PluginDef } from './components/SettingsField';

import { DisplayView } from './views/DisplayView';
import { StorageView } from './views/StorageView';
import { AuthView } from './views/AuthView';
import { OutputView } from './views/OutputView';

import './index.css';
import './overlay.css';

const formatTime = (ms: number) => {
  if (isNaN(ms) || ms <= 0) return '0:00';
  const totalSecs = Math.floor(ms / 1000);
  const mins = Math.floor(totalSecs / 60);
  const secs = totalSecs % 60;
  return `${mins}:${secs < 10 ? '0' : ''}${secs}`;
};

function App() {
  const [activeView, setActiveView] = useState<View>('display');
  const [activeOverlay, setActiveOverlay] = useState<string>('now_playing');
  const [playback, setPlayback] = useState<PlaybackState>({
    title: "Resonance",
    artist: "Home",
    album: "Odyssey",
    art_source: "https://i.scdn.co/image/ab67616d0000b27370364408e063f2f0c76f4e17",
    progress_ms: 165000,
    duration_ms: 210000,
    is_playing: true,
  });

  const [displayProgressMs, setDisplayProgressMs] = useState<number>(165000);
  const [lastUpdated, setLastUpdated] = useState<number>(Date.now());
  const [updateVersion, setUpdateVersion] = useState<string | null>(null);
  const [updateUrl, setUpdateUrl] = useState<string | null>(null);
  const [template, setTemplate] = useState('Now Playing: {title} by {artist}');
  const [overlaysCss, setOverlaysCss] = useState<Record<string, string>>({});
  const [plugins, setPlugins] = useState<PluginDef[]>([]);
  const [pluginSettings, setPluginSettings] = useState<Record<string, Record<string, any>>>({});
  const [notPlayingSettings, setNotPlayingSettings] = useState({
    not_playing_title: 'Not Playing',
    not_playing_artist: 'Unknown Artist',
    not_playing_album: 'Unknown Album'
  });

  const loadSettings = async () => {
    try {
      const metadata = await invoke<PluginDef[]>('get_plugins_metadata');
      setPlugins(metadata);

      const savedTemplate = await invoke<string | null>('get_output_template');
      if (savedTemplate !== null) {
        setTemplate(savedTemplate);
      }
      
      const updateStatus = await invoke<any>('get_update_status');
      if (updateStatus) {
        setUpdateVersion(updateStatus.tag_name);
        setUpdateUrl(updateStatus.html_url);
      }

      const npSettings = await invoke<any>('get_not_playing_settings');
      setNotPlayingSettings(npSettings);

      // Fetch settings for each plugin
      const settingsMap: Record<string, Record<string, any>> = {};
      const cssMap: Record<string, string> = {};
      
      const coreOverlays = ['now_playing', ...metadata.filter(p => p.category === 'overlay').map(p => p.id)];
      for (const id of coreOverlays) {
        try {
          const cssValue = await invoke<string | null>('get_overlay_css', { id });
          if (cssValue) cssMap[id] = cssValue;
        } catch (err) {
          console.error(`Failed to load CSS for overlay ${id}:`, err);
        }
      }
      setOverlaysCss(cssMap);

      for (const p of metadata) {
        try {
          const s = await invoke<Record<string, any>>('get_plugin_settings', { pluginId: p.id });
          settingsMap[p.id] = s;
        } catch (err) {
          console.error(`Failed to load settings for plugin ${p.id}:`, err);
        }
      }
      setPluginSettings(settingsMap);
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  };

  // Load initial settings on mount
  useEffect(() => {
    loadSettings();
  }, []);

  // Listen for auth_success event from Rust
  useEffect(() => {
    const unlisten = listen<string>('auth_success', (event) => {
      console.log(`Auth success event received for provider: ${event.payload}`);
      loadSettings();
    });
    return () => {
      unlisten.then(f => f());
    };
  }, []);

  // Listen for update_available event
  useEffect(() => {
    const unlisten = listen<any>('update_available', (event) => {
      setUpdateVersion(event.payload.tag_name);
      setUpdateUrl(event.payload.html_url);
    });
    return () => {
      unlisten.then(f => f());
    };
  }, []);

  // Listen for playback updates from Rust
  useEffect(() => {
    const unlisten = listen<PlaybackState>('playback_update', (event) => {
      setPlayback(event.payload);
      setLastUpdated(Date.now());
      setDisplayProgressMs(event.payload.progress_ms);
    });
    return () => {
      unlisten.then(f => f());
    };
  }, []);

  // Smooth playback progress interpolation
  useEffect(() => {
    if (!playback.is_playing) {
      setDisplayProgressMs(playback.progress_ms);
      return;
    }
    const interval = setInterval(() => {
      const elapsed = Date.now() - lastUpdated;
      const current = Math.min(playback.progress_ms + elapsed, playback.duration_ms);
      setDisplayProgressMs(current);
    }, 30);
    return () => clearInterval(interval);
  }, [playback.is_playing, playback.progress_ms, playback.duration_ms, lastUpdated]);

  const progressPercent = (playback && playback.duration_ms > 0)
    ? (displayProgressMs / playback.duration_ms) * 100
    : 0;

  const handleOpenUpdate = async () => {
    if (updateUrl) {
      try {
        await openUrl(updateUrl);
      } catch (e) {
        console.error('Failed to open update link:', e);
      }
    }
  };

  const updatePluginSetting = async (pluginId: string, key: string, value: any) => {
    const newSettings = { ...pluginSettings[pluginId], [key]: value };
    setPluginSettings({ ...pluginSettings, [pluginId]: newSettings });
    try {
      await invoke('set_plugin_settings', { pluginId, newSettings });
    } catch (e) {
      console.error('Failed to save plugin settings:', e);
    }
  };

  const updateOverlayCss = async (id: string, css: string) => {
    setOverlaysCss(prev => ({ ...prev, [id]: css }));
    try {
      await invoke('set_overlay_css', { id, css });
    } catch (err) {
      console.error(`Failed to save CSS for overlay ${id}:`, err);
    }
  };

  const triggerAction = async (pluginId: string, actionName: string) => {
    try {
      await invoke<any>('trigger_plugin_action', { pluginId, actionName });
      
      // Force reload plugin settings to reflect any side-effects
      const updatedSettings = await invoke<Record<string, any>>('get_plugin_settings', { pluginId });
      setPluginSettings((prev) => ({ ...prev, [pluginId]: updatedSettings }));
    } catch (e) {
      console.error(`Failed to trigger action '${actionName}' for plugin '${pluginId}':`, e);
      alert(`Error: ${e}`);
    }
  };

  const updateNotPlayingSetting = async (key: string, value: string) => {
    const newSettings = { ...notPlayingSettings, [key]: value };
    setNotPlayingSettings(newSettings);
    try {
      await invoke('set_not_playing_settings', newSettings);
    } catch (e) {
      console.error('Failed to save not playing settings:', e);
    }
  };

  const renderView = () => {
    switch (activeView) {
      case 'display':
        return (
          <DisplayView
            playback={playback}
            progressPercent={progressPercent}
            displayProgressMs={displayProgressMs}
            formatTime={formatTime}
            overlaysCss={overlaysCss}
            setOverlaysCss={updateOverlayCss}
            plugins={plugins}
            pluginSettings={pluginSettings}
            notPlayingSettings={notPlayingSettings}
            activeOverlay={activeOverlay}
            setActiveOverlay={setActiveOverlay}
            updatePluginSetting={updatePluginSetting}
            updateNotPlayingSetting={updateNotPlayingSetting}
            triggerAction={triggerAction}
          />
        );
      case 'storage':
        return (
          <StorageView
            plugins={plugins}
            pluginSettings={pluginSettings}
            updatePluginSetting={updatePluginSetting}
            triggerAction={triggerAction}
          />
        );
      case 'auth':
        return (
          <AuthView
            plugins={plugins}
            pluginSettings={pluginSettings}
            updatePluginSetting={updatePluginSetting}
            triggerAction={triggerAction}
          />
        );
      case 'output':
        return (
          <OutputView
            template={template}
            setTemplate={setTemplate}
            playback={playback}
          />
        );
    }
  };

  return (
    <div className="app-container">
      <Sidebar
        activeView={activeView}
        setActiveView={setActiveView}
        updateVersion={updateVersion}
        handleOpenUpdate={handleOpenUpdate}
      />
      <main className="content">
        {renderView()}
      </main>
    </div>
  );
}

export default App;
