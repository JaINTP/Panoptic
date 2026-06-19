import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { openPath } from '@tauri-apps/plugin-opener';
import { SettingsField, PluginDef } from '../components/SettingsField';

interface StorageViewProps {
  plugins: PluginDef[];
  pluginSettings: Record<string, Record<string, any>>;
  updatePluginSetting: (pluginId: string, key: string, value: any) => void;
  triggerAction: (pluginId: string, actionName: string) => void;
}

interface StoragePaths {
  config_dir: string;
  artwork_dir: string;
}

export const StorageView: React.FC<StorageViewProps> = ({
  plugins,
  pluginSettings,
  updatePluginSetting,
  triggerAction,
}) => {
  const [paths, setPaths] = useState<StoragePaths | null>(null);

  useEffect(() => {
    invoke<StoragePaths>('get_storage_paths').then(setPaths).catch(console.error);
  }, []);

  const storagePlugins = plugins.filter(
    (p) => (p.category === 'storage' || !p.category) && p.fields.length > 0
  );

  const handleBrowse = async (path: string) => {
    if (!path) return;
    try {
      await openPath(path);
    } catch (e) {
      console.error('Failed to open path:', path, e);
    }
  };

  return (
    <div className="view-pane view-pane-scrollable">
      <h1 className="view-title">Storage & Environment</h1>

      {storagePlugins.map((plugin) => (
        <div key={plugin.id} className="section">
          <h2 className="section-title">{plugin.name}</h2>
          <div className="settings-card">
            {plugin.fields.map((field) => (
              <SettingsField
                key={field.key}
                field={field}
                category={plugin.category}
                currentValue={pluginSettings[plugin.id]?.[field.key]}
                onUpdate={(key, value) => updatePluginSetting(plugin.id, key, value)}
                onTriggerAction={(actionName) => triggerAction(plugin.id, actionName)}
              />
            ))}
          </div>
        </div>
      ))}

      <div className="section">
        <h2 className="section-title">System Paths</h2>
        <div className="settings-card">
          <div className="row" style={{ marginBottom: '16px' }}>
            <div className="label-container" style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
              <div className="label">Artwork Cache</div>
              <span style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
                Location where album art is cached. Click Browse to open in file explorer.
              </span>
            </div>
            <div className="input-group" style={{ flex: 1, justifyContent: 'flex-end' }}>
              <input
                type="text"
                readOnly
                value={paths?.artwork_dir ?? 'Loading...'}
                style={{ width: '260px' }}
              />
              <button
                type="button"
                className="btn-secondary"
                onClick={() => paths?.artwork_dir && handleBrowse(paths.artwork_dir)}
                disabled={!paths?.artwork_dir}
              >
                Browse...
              </button>
            </div>
          </div>

          <div className="row">
            <div className="label-container" style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
              <div className="label">Config Directory</div>
              <span style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
                App configuration and current_track.txt are stored here.
              </span>
            </div>
            <div className="input-group" style={{ flex: 1, justifyContent: 'flex-end' }}>
              <input
                type="text"
                readOnly
                value={paths?.config_dir ?? 'Loading...'}
                style={{ width: '260px' }}
              />
              <button
                type="button"
                className="btn-secondary"
                onClick={() => paths?.config_dir && handleBrowse(paths.config_dir)}
                disabled={!paths?.config_dir}
              >
                Browse...
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
