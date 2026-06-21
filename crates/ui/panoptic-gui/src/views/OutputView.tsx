import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { PlaybackState } from '../components/OverlayPreview';
import { PlaceholderGrid } from '../components/PlaceholderGrid';
import { SettingsField, PluginDef } from '../components/SettingsField';

interface ObsAudioSource {
  name: string;
  muted: boolean;
}

interface ObsSceneItem {
  id: number;
  name: string;
  enabled: boolean;
}

interface ObsStatus {
  connected: boolean;
  current_scene: string;
  scenes: string[];
  audio_sources: ObsAudioSource[];
  scene_items: ObsSceneItem[];
  error: string | null;
}

interface OutputViewProps {
  template: string;
  setTemplate: (val: string) => void;
  playback: PlaybackState;
  plugins: PluginDef[];
  pluginSettings: Record<string, Record<string, any>>;
  updatePluginSetting: (pluginId: string, key: string, value: any) => void;
  triggerAction: (pluginId: string, actionName: string) => void;
}

export const OutputView: React.FC<OutputViewProps> = ({
  template,
  setTemplate,
  playback,
  plugins,
  pluginSettings,
  updatePluginSetting,
  triggerAction,
}) => {
  const [obsStatus, setObsStatus] = useState<ObsStatus>({
    connected: false,
    current_scene: '',
    scenes: [],
    audio_sources: [],
    scene_items: [],
    error: null,
  });

  useEffect(() => {
    invoke<ObsStatus>('get_obs_status')
      .then(setObsStatus)
      .catch(() => {});

    const unlisten = listen<ObsStatus>('obs_status', (e) => {
      setObsStatus(e.payload);
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  const outputPlugins = plugins.filter((p) => p.category === 'output');

  const getFormatTime = (ms: number) => {
    if (isNaN(ms) || ms <= 0) return '0:00';
    const totalSecs = Math.floor(ms / 1000);
    const hours = Math.floor(totalSecs / 3600);
    const mins = Math.floor((totalSecs % 3600) / 60);
    const secs = totalSecs % 60;
    const pad = (num: number) => num.toString().padStart(2, '0');
    if (hours > 0) {
      return `${hours}:${pad(mins)}:${pad(secs)}`;
    }
    return `${mins}:${pad(secs)}`;
  };

  const getComponents = (ms: number) => {
    const totalSecs = Math.floor((ms || 0) / 1000);
    const h = Math.floor(totalSecs / 3600);
    const m = Math.floor((totalSecs % 3600) / 60);
    const s = totalSecs % 60;
    const mTotal = Math.floor(totalSecs / 60);
    const pad = (num: number) => num.toString().padStart(2, '0');
    return {
      h: h.toString(),
      m: pad(m),
      s: pad(s),
      mRaw: m.toString(),
      sRaw: s.toString(),
      mTotal: mTotal.toString(),
      mTotalPadded: pad(mTotal),
      sTotal: totalSecs.toString(),
    };
  };

  const pComp = getComponents(playback.progress_ms);
  const dComp = getComponents(playback.duration_ms);
  const pFormatted = getFormatTime(playback.progress_ms);
  const dFormatted = getFormatTime(playback.duration_ms);

  const formattedOutput = template
    .replace(/{title}/g, playback.title || '')
    .replace(/{artist}/g, playback.artist || '')
    .replace(/{album}/g, playback.album || '')
    .replace(/{progress_ms}/g, String(playback.progress_ms || 0))
    .replace(/{duration_ms}/g, String(playback.duration_ms || 0))
    .replace(/{progress}/g, pFormatted)
    .replace(/{duration}/g, dFormatted)
    .replace(/{progress_h}/g, pComp.h)
    .replace(/{progress_m}/g, pComp.m)
    .replace(/{progress_s}/g, pComp.s)
    .replace(/{progress_m_raw}/g, pComp.mRaw)
    .replace(/{progress_s_raw}/g, pComp.sRaw)
    .replace(/{progress_m_total}/g, pComp.mTotal)
    .replace(/{progress_m_total_padded}/g, pComp.mTotalPadded)
    .replace(/{progress_s_total}/g, pComp.sTotal)
    .replace(/{duration_h}/g, dComp.h)
    .replace(/{duration_m}/g, dComp.m)
    .replace(/{duration_s}/g, dComp.s)
    .replace(/{duration_m_raw}/g, dComp.mRaw)
    .replace(/{duration_s_raw}/g, dComp.sRaw)
    .replace(/{duration_m_total}/g, dComp.mTotal)
    .replace(/{duration_m_total_padded}/g, dComp.mTotalPadded)
    .replace(/{duration_s_total}/g, dComp.sTotal);

  const insertPlaceholder = (placeholder: string) => {
    const textarea = document.querySelector('.code-editor') as HTMLTextAreaElement;
    if (textarea) {
      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      const text = textarea.value;
      const before = text.substring(0, start);
      const after = text.substring(end, text.length);
      const newVal = before + placeholder + after;
      setTemplate(newVal);
      invoke('set_output_template', { template: newVal }).catch((err) =>
        console.error('Failed to save template:', err)
      );
      setTimeout(() => {
        textarea.focus();
        textarea.selectionStart = textarea.selectionEnd = start + placeholder.length;
      }, 0);
    } else {
      const newVal = template + placeholder;
      setTemplate(newVal);
      invoke('set_output_template', { template: newVal }).catch((err) =>
        console.error('Failed to save template:', err)
      );
    }
  };

  const handleSwitchScene = (scene: string) => {
    triggerAction('obs-websocket', `switch_scene:${scene}`);
  };

  const handleToggleMute = (name: string) => {
    triggerAction('obs-websocket', `toggle_mute:${name}`);
  };

  const handleToggleSceneItem = (id: number, currentEnabled: boolean) => {
    triggerAction('obs-websocket', `toggle_scene_item:${id}:${!currentEnabled}`);
  };

  const sectionLabelStyle: React.CSSProperties = {
    fontSize: '11px',
    fontWeight: 600,
    textTransform: 'uppercase',
    letterSpacing: '0.06em',
    color: 'var(--text-muted)',
    marginBottom: '6px',
  };

  return (
    <div className="view-pane view-pane-scrollable">
      <h1 className="view-title">Output</h1>

      {/* Output-category plugin settings */}
      {outputPlugins.map((plugin) => (
        <div key={plugin.id} className="section">
          <h2 className="section-title">{plugin.name}</h2>
          <div className="settings-card">
            {plugin.fields.map((field) => (
              <SettingsField
                key={field.key}
                field={field}
                category={plugin.category}
                currentValue={pluginSettings[plugin.id]?.[field.key]}
                accessTokenExists={!!pluginSettings[plugin.id]?.access_token}
                onUpdate={(key, value) => updatePluginSetting(plugin.id, key, value)}
                onTriggerAction={(actionName) => triggerAction(plugin.id, actionName)}
              />
            ))}
          </div>

          {/* OBS-specific: connection status, scenes, audio, and source visibility */}
          {plugin.id === 'obs-websocket' && (
            <div style={{ marginTop: '12px', display: 'flex', flexDirection: 'column', gap: '12px' }}>

              {/* Status bar */}
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '8px',
                  padding: '8px 12px',
                  borderRadius: '6px',
                  background: 'var(--bg-card)',
                  border: '1px solid var(--border)',
                }}
              >
                <span
                  style={{
                    width: '8px',
                    height: '8px',
                    borderRadius: '50%',
                    flexShrink: 0,
                    background: obsStatus.connected
                      ? 'var(--accent-primary)'
                      : obsStatus.error
                      ? '#e05555'
                      : 'var(--text-muted)',
                  }}
                />
                <span style={{ fontSize: '13px', color: 'var(--text-main)' }}>
                  {obsStatus.connected
                    ? obsStatus.current_scene
                      ? `Connected — ${obsStatus.current_scene}`
                      : 'Connected'
                    : obsStatus.error ?? 'Disconnected'}
                </span>
              </div>

              {/* Scene switcher */}
              {obsStatus.scenes.length > 0 && (
                <div>
                  <div style={sectionLabelStyle}>Scenes</div>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                    {obsStatus.scenes.map((scene) => {
                      const isActive = scene === obsStatus.current_scene;
                      return (
                        <button
                          key={scene}
                          type="button"
                          onClick={() => !isActive && handleSwitchScene(scene)}
                          style={{
                            display: 'flex',
                            alignItems: 'center',
                            gap: '8px',
                            padding: '7px 10px',
                            borderRadius: '6px',
                            border: isActive
                              ? '1px solid var(--accent-primary)'
                              : '1px solid var(--border)',
                            background: isActive ? 'rgba(139, 92, 246, 0.08)' : 'var(--bg-card)',
                            color: isActive ? 'var(--accent-primary)' : 'var(--text-main)',
                            fontSize: '13px',
                            cursor: isActive ? 'default' : 'pointer',
                            textAlign: 'left',
                            width: '100%',
                          }}
                        >
                          <span
                            style={{
                              width: '6px',
                              height: '6px',
                              borderRadius: '50%',
                              flexShrink: 0,
                              background: isActive ? 'var(--accent-primary)' : 'transparent',
                              border: isActive ? 'none' : '1px solid var(--border)',
                            }}
                          />
                          {scene}
                        </button>
                      );
                    })}
                  </div>
                </div>
              )}

              {/* Audio source mute controls */}
              {obsStatus.audio_sources.length > 0 && (
                <div>
                  <div style={sectionLabelStyle}>Audio Sources</div>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                    {obsStatus.audio_sources.map((src) => (
                      <div
                        key={src.name}
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'space-between',
                          padding: '7px 10px',
                          borderRadius: '6px',
                          border: '1px solid var(--border)',
                          background: 'var(--bg-card)',
                        }}
                      >
                        <span
                          style={{
                            fontSize: '13px',
                            color: src.muted ? 'var(--text-muted)' : 'var(--text-main)',
                            overflow: 'hidden',
                            textOverflow: 'ellipsis',
                            whiteSpace: 'nowrap',
                            flex: 1,
                            minWidth: 0,
                          }}
                        >
                          {src.name}
                        </span>
                        <button
                          type="button"
                          onClick={() => handleToggleMute(src.name)}
                          style={{
                            flexShrink: 0,
                            marginLeft: '10px',
                            padding: '3px 10px',
                            borderRadius: '4px',
                            border: '1px solid var(--border)',
                            background: src.muted ? 'rgba(224, 85, 85, 0.12)' : 'var(--bg-hover)',
                            color: src.muted ? '#e05555' : 'var(--text-main)',
                            fontSize: '12px',
                            cursor: 'pointer',
                            fontWeight: 500,
                          }}
                        >
                          {src.muted ? 'Unmute' : 'Mute'}
                        </button>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Scene source visibility toggles */}
              {obsStatus.scene_items.length > 0 && (
                <div>
                  <div style={sectionLabelStyle}>
                    Sources — {obsStatus.current_scene}
                  </div>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                    {obsStatus.scene_items.map((item) => (
                      <div
                        key={item.id}
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'space-between',
                          padding: '7px 10px',
                          borderRadius: '6px',
                          border: '1px solid var(--border)',
                          background: 'var(--bg-card)',
                          opacity: item.enabled ? 1 : 0.5,
                        }}
                      >
                        <span
                          style={{
                            fontSize: '13px',
                            color: 'var(--text-main)',
                            overflow: 'hidden',
                            textOverflow: 'ellipsis',
                            whiteSpace: 'nowrap',
                            flex: 1,
                            minWidth: 0,
                          }}
                        >
                          {item.name}
                        </span>
                        <button
                          type="button"
                          onClick={() => handleToggleSceneItem(item.id, item.enabled)}
                          style={{
                            flexShrink: 0,
                            marginLeft: '10px',
                            padding: '3px 10px',
                            borderRadius: '4px',
                            border: '1px solid var(--border)',
                            background: item.enabled
                              ? 'rgba(139, 92, 246, 0.08)'
                              : 'var(--bg-hover)',
                            color: item.enabled ? 'var(--accent-primary)' : 'var(--text-muted)',
                            fontSize: '12px',
                            cursor: 'pointer',
                            fontWeight: 500,
                          }}
                        >
                          {item.enabled ? 'Visible' : 'Hidden'}
                        </button>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      ))}

      {/* Text output templating */}
      <div className="section" style={{ marginBottom: '16px' }}>
        <h2 className="section-title">Template String</h2>
        <textarea
          className="code-editor"
          style={{ height: '80px', marginBottom: '8px' }}
          value={template}
          onChange={(e) => {
            const val = e.target.value;
            setTemplate(val);
            invoke('set_output_template', { template: val }).catch((err) =>
              console.error('Failed to save template:', err)
            );
          }}
        />
      </div>

      <div className="section" style={{ marginBottom: '16px' }}>
        <h2 className="section-title">Available Placeholders (Click to Insert)</h2>
        <PlaceholderGrid onInsertPlaceholder={insertPlaceholder} />
      </div>

      <div className="section" style={{ marginBottom: '16px' }}>
        <h2 className="section-title">Resulting Message Preview</h2>
        <div
          style={{
            padding: '16px',
            borderRadius: '8px',
            border: '1px solid var(--border)',
            backgroundColor: 'var(--bg-card)',
            fontFamily: 'monospace',
            fontSize: '14px',
            color: 'var(--text-main)',
            minHeight: '50px',
            display: 'flex',
            alignItems: 'center',
          }}
        >
          {formattedOutput || (
            <span style={{ color: 'var(--text-muted)' }}>(Empty template output)</span>
          )}
        </div>
      </div>
    </div>
  );
};
