import React from 'react';
import { SettingsField, PluginDef } from '../components/SettingsField';

interface StorageViewProps {
  plugins: PluginDef[];
  pluginSettings: Record<string, Record<string, any>>;
  updatePluginSetting: (pluginId: string, key: string, value: any) => void;
  triggerAction: (pluginId: string, actionName: string) => void;
}

export const StorageView: React.FC<StorageViewProps> = ({
  plugins,
  pluginSettings,
  updatePluginSetting,
  triggerAction,
}) => {
  const storagePlugins = plugins.filter(
    (p) => (p.category === 'storage' || !p.category) && p.fields.length > 0
  );

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
                Location where downloaded album covers are cached.
              </span>
            </div>
            <div className="input-group" style={{ flex: 1, justifyContent: 'flex-end' }}>
              <input type="text" readOnly value="/home/user/.cache/panoptic/artworks" style={{ width: '260px' }} />
              <button type="button" className="btn-secondary">Browse...</button>
            </div>
          </div>

          <div className="row">
            <div className="label-container" style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
              <div className="label">OpenAPI Spec Store</div>
              <span style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
                Target directory for schema ingestion specs.
              </span>
            </div>
            <div className="input-group" style={{ flex: 1, justifyContent: 'flex-end' }}>
              <input type="text" readOnly value="/home/user/.config/panoptic/schema" style={{ width: '260px' }} />
              <button type="button" className="btn-secondary">Browse...</button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
