import React from 'react';
import { SettingsField, PluginDef } from '../components/SettingsField';

interface IntegrationsViewProps {
  plugins: PluginDef[];
  pluginSettings: Record<string, Record<string, any>>;
  updatePluginSetting: (pluginId: string, key: string, value: any) => void;
  triggerAction: (pluginId: string, actionName: string) => void;
}

export const IntegrationsView: React.FC<IntegrationsViewProps> = ({
  plugins,
  pluginSettings,
  updatePluginSetting,
  triggerAction,
}) => {
  const integrationsPlugins = plugins.filter((p) => p.category === 'general');

  return (
    <div className="view-pane view-pane-scrollable">
      <h1 className="view-title">Integrations</h1>
      {integrationsPlugins.map((plugin) => (
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
      {integrationsPlugins.length === 0 && (
        <div style={{ color: 'var(--text-secondary)', padding: '20px', textAlign: 'center' }}>
          No integrations plugins registered.
        </div>
      )}
    </div>
  );
};
