import React from 'react';

export interface SettingFieldType {
  type: 'Text' | 'Password' | 'Number' | 'Boolean' | 'Select' | 'Action';
  options?: {
    options?: string[];
    button_label?: string;
    action_name?: string;
  };
}

export interface SettingFieldDef {
  key: string;
  label: string;
  description: string | null;
  field_type: SettingFieldType;
  default_value: any;
}

export interface PluginDef {
  id: string;
  name: string;
  category: 'auth' | 'overlay' | 'output' | 'storage' | 'general' | null;
  fields: SettingFieldDef[];
}

interface SettingsFieldProps {
  field: SettingFieldDef;
  category: string | null;
  currentValue: any;
  accessTokenExists?: boolean;
  onUpdate: (key: string, value: any) => void;
  onTriggerAction: (actionName: string) => void;
}

export const SettingsField: React.FC<SettingsFieldProps> = ({
  field,
  category,
  currentValue,
  accessTokenExists,
  onUpdate,
  onTriggerAction,
}) => {
  const value = currentValue ?? (typeof field.default_value === 'string' ? field.default_value : '');

  return (
    <div className="row" style={{ marginBottom: '16px' }}>
      <div className="label-container" style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <div className="label">{field.label}</div>
          {category === 'auth' && field.field_type.type === 'Action' && accessTokenExists && (
            <span className="badge badge-emerald" style={{ marginTop: 0 }}>Connected</span>
          )}
        </div>
        {field.description && (
          <span style={{ fontSize: '11px', color: 'var(--text-muted)', maxWidth: '300px' }}>
            {field.description}
          </span>
        )}
      </div>

      <div className="input-group" style={{ flex: 1, justifyContent: 'flex-end' }}>
        {field.field_type.type === 'Text' && (
          <input
            type="text"
            value={value}
            onChange={(e) => onUpdate(field.key, e.target.value)}
            placeholder={typeof field.default_value === 'string' ? field.default_value : ''}
          />
        )}
        {field.field_type.type === 'Number' && (
          <input
            type="number"
            value={currentValue ?? field.default_value}
            onChange={(e) => onUpdate(field.key, parseFloat(e.target.value))}
          />
        )}
        {field.field_type.type === 'Password' && (
          <input
            type="password"
            value={value}
            onChange={(e) => onUpdate(field.key, e.target.value)}
          />
        )}
        {field.field_type.type === 'Boolean' && (
          <input
            type="checkbox"
            checked={!!(currentValue ?? field.default_value)}
            onChange={(e) => onUpdate(field.key, e.target.checked)}
          />
        )}
        {field.field_type.type === 'Action' && (
          (() => {
            let label = field.field_type.options?.button_label || 'Action';
            let action = field.field_type.options?.action_name || '';

            // Toggle Link/Unlink UI button styles for auth plugins
            if (category === 'auth' && action === 'link' && accessTokenExists) {
              label = 'Unlink Account';
              action = 'unlink';
            }

            return (
              <button
                type="button"
                className={action === 'unlink' ? "btn-secondary" : "btn-primary"}
                onClick={() => onTriggerAction(action)}
              >
                {label}
              </button>
            );
          })()
        )}
      </div>
    </div>
  );
};
