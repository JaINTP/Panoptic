import React, { useState, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import CodeMirror from '@uiw/react-codemirror';
import { css, cssLanguage } from '@codemirror/lang-css';
import { linter, lintGutter, Diagnostic } from '@codemirror/lint';
import { syntaxTree } from '@codemirror/language';
import { CompletionContext, CompletionResult } from '@codemirror/autocomplete';
import { OverlayPreview, PlaybackState } from '../components/OverlayPreview';
import { HypeTrainPreview, HypeTrainState } from '../components/HypeTrainPreview';
import { TwitchAlertPreview, AlertState } from '../components/TwitchAlertPreview';
import { SettingsField, PluginDef } from '../components/SettingsField';
import { TwitchAlertPlaceholderGrid } from '../components/TwitchAlertPlaceholderGrid';

// Autocomplete and Lint helpers
const PANOPTIC_CLASSES = [
  { label: '.panoptic-overlay-wrapper', type: 'class', detail: 'Overlay container wrapper' },
  { label: '.panoptic-overlay-card', type: 'class', detail: 'The main display card' },
  { label: '.panoptic-overlay-art-container', type: 'class', detail: 'Art container wrapper' },
  { label: '.panoptic-overlay-album-art', type: 'class', detail: 'Album artwork image element' },
  { label: '.panoptic-overlay-text-container', type: 'class', detail: 'Info text container' },
  { label: '.panoptic-overlay-track-title', type: 'class', detail: 'Main title text' },
  { label: '.panoptic-overlay-track-artist', type: 'class', detail: 'Sub-title/artist text' },
  { label: '.panoptic-overlay-track-album', type: 'class', detail: 'Secondary info/album text' },
  { label: '.panoptic-overlay-progress-section', type: 'class', detail: 'Progress bar section' },
  { label: '.panoptic-overlay-progress-container', type: 'class', detail: 'Progress bar track' },
  { label: '.panoptic-overlay-progress-bar', type: 'class', detail: 'Progress bar fill' },
  { label: '.panoptic-overlay-time-display', type: 'class', detail: 'Time display container' },
  { label: '.panoptic-overlay-time-separator', type: 'class', detail: 'Time separator element' },
  { label: '.hype-train-card', type: 'class', detail: 'Hype Train main card' },
  { label: '.hype-idle-state', type: 'class', detail: 'Inactive overlay state' },
  { label: '.hype-active-state', type: 'class', detail: 'Active event state' },
  { label: '.hype-leaderboard-list', type: 'class', detail: 'Leaderboard list' },
  { label: '.hype-leaderboard-item', type: 'class', detail: 'Leaderboard entry' },
  { label: '.status-icon', type: 'class', detail: 'Status indicator icon' },
  { label: '.alert-card', type: 'class', detail: 'Alert notification card' },
  { label: '.alert-node', type: 'class', detail: 'Alert stack entry' },
  { label: '.alert-text-content', type: 'class', detail: 'Text within an alert' }
];

const PANOPTIC_VARS = [
  { label: '--panoptic-overlay-card-display', type: 'variable', detail: 'flex | block | grid' },
  { label: '--panoptic-overlay-card-gap', type: 'variable', detail: 'Gap size' },
  { label: '--panoptic-overlay-card-background', type: 'variable', detail: 'Main background' },
  { label: '--panoptic-overlay-card-border-style', type: 'variable', detail: 'solid | none | dashed' },
  { label: '--panoptic-overlay-card-border-width', type: 'variable', detail: 'Border width' },
  { label: '--panoptic-overlay-card-border-color', type: 'variable', detail: 'Border color' },
  { label: '--panoptic-overlay-card-backdrop-blur', type: 'variable', detail: 'Backdrop blur' },
  { label: '--panoptic-overlay-card-padding', type: 'variable', detail: 'Card padding' },
  { label: '--panoptic-overlay-card-border-radius', type: 'variable', detail: 'Corner radius' },
  { label: '--panoptic-overlay-card-width', type: 'variable', detail: 'Card width' },
  { label: '--panoptic-overlay-card-box-shadow', type: 'variable', detail: 'Card box shadow' },
  { label: '--ht-color-bg', type: 'variable', detail: 'Event background' },
  { label: '--ht-color-primary', type: 'variable', detail: 'Event primary color' },
  { label: '--ht-color-secondary', type: 'variable', detail: 'Event secondary color' },
  { label: '--container-bottom', type: 'variable', detail: 'Stack position bottom' },
  { label: '--container-right', type: 'variable', detail: 'Stack position right' },
  { label: '--stack-direction', type: 'variable', detail: 'column | column-reverse' },
  { label: '--stack-gap', type: 'variable', detail: 'Space between alerts' },
  { label: '--alert-duration', type: 'variable', detail: 'Time shown (e.g. 8s)' }
];

function panopticCssAutocomplete(context: CompletionContext): CompletionResult | null {
  const classWord = context.matchBefore(/\.[a-zA-Z0-9_-]*/);
  if (classWord) {
    if (classWord.from === classWord.to && !context.explicit) return null;
    return {
      from: classWord.from,
      options: PANOPTIC_CLASSES
    };
  }

  const varWord = context.matchBefore(/-[a-zA-Z0-9_-]*/);
  if (varWord) {
    if (varWord.from === varWord.to && !context.explicit) return null;
    return {
      from: varWord.from,
      options: PANOPTIC_VARS
    };
  }

  return null;
}

const customCompletionExtension = cssLanguage.data.of({
  autocomplete: panopticCssAutocomplete
});

const cssSyntaxLinter = linter((view) => {
  const diagnostics: Diagnostic[] = [];
  syntaxTree(view.state).cursor().iterate((node) => {
    if (node.type.isError) {
      diagnostics.push({
        from: node.from,
        to: node.to,
        severity: 'error',
        message: 'CSS syntax error'
      });
    }
  });
  return diagnostics;
});

interface DisplayViewProps {
  playback: PlaybackState;
  progressPercent: number;
  displayProgressMs: number;
  formatTime: (ms: number) => string;
  overlaysCss: Record<string, string>;
  setOverlaysCss: (id: string, css: string) => void;
  plugins: PluginDef[];
  pluginSettings: Record<string, Record<string, any>>;
  notPlayingSettings: Record<string, string>;
  activeOverlay: string;
  setActiveOverlay: (id: string) => void;
  updatePluginSetting: (pluginId: string, key: string, value: any) => void;
  updateNotPlayingSetting: (key: string, value: string) => void;
  triggerAction: (pluginId: string, actionName: string) => void;
}

export const DisplayView: React.FC<DisplayViewProps> = ({
  playback,
  progressPercent,
  displayProgressMs,
  formatTime,
  overlaysCss,
  setOverlaysCss,
  plugins,
  pluginSettings,
  notPlayingSettings,
  activeOverlay,
  setActiveOverlay,
  updatePluginSetting,
  updateNotPlayingSetting,
  triggerAction,
}) => {
  const [hypeTrainState, setHypeTrainState] = useState<HypeTrainState>({
    active: false,
    level: 1,
    total: 0,
    progress: 0,
    goal: 100,
    top_contributions: [],
    started_at: '',
    expires_at: '',
  });

  const [alertState, setAlertState] = useState<AlertState>({
    active_alerts: [],
  });

  const focusedInputRef = useRef<{ pluginId: string, key: string } | null>(null);

  useEffect(() => {
    const unlistenHype = listen<HypeTrainState>('twitch_hype_train', (event) => {
      setHypeTrainState(event.payload);
    });
    const unlistenHypeClear = listen('twitch_hype_train_clear', () => {
      setHypeTrainState(prev => ({ ...prev, active: false }));
    });
    const unlistenAlert = listen<AlertState>('twitch_alert', (event) => {
      setAlertState(event.payload);
    });

    return () => {
      unlistenHype.then(f => f());
      unlistenHypeClear.then(f => f());
      unlistenAlert.then(f => f());
    };
  }, []);

  const overlayTabs = [
    { id: 'now_playing', name: 'Now Playing' },
    ...plugins
      .filter((p) => p.category === 'overlay')
      .map((p) => ({ id: p.id, name: p.name }))
  ];

  const handleInsertAlertPlaceholder = (placeholder: string) => {
    if (!focusedInputRef.current) return;
    const { pluginId, key } = focusedInputRef.current;
    const currentVal = pluginSettings[pluginId]?.[key] || '';
    updatePluginSetting(pluginId, key, currentVal + placeholder);
  };

  const renderTextSettings = () => {
    if (activeOverlay === 'now_playing') {
      return (
        <div className="section" style={{ margin: 0 }}>
          <h2 className="section-title">Fallback Text</h2>
          <div className="settings-card" style={{ border: 'none', background: 'transparent', padding: 0 }}>
            <SettingsField
              field={{
                key: 'not_playing_title',
                label: 'Fallback Title',
                description: 'Shown when no track is playing',
                field_type: { type: 'Text' },
                default_value: 'Not Playing'
              }}
              category="overlay"
              currentValue={notPlayingSettings.not_playing_title}
              onUpdate={(key, val) => updateNotPlayingSetting(key, val)}
              onTriggerAction={() => {}}
            />
            <SettingsField
              field={{
                key: 'not_playing_artist',
                label: 'Fallback Artist',
                description: 'Shown when no track is playing',
                field_type: { type: 'Text' },
                default_value: 'Unknown Artist'
              }}
              category="overlay"
              currentValue={notPlayingSettings.not_playing_artist}
              onUpdate={(key, val) => updateNotPlayingSetting(key, val)}
              onTriggerAction={() => {}}
            />
            <SettingsField
              field={{
                key: 'not_playing_album',
                label: 'Fallback Album',
                description: 'Shown when no track is playing',
                field_type: { type: 'Text' },
                default_value: 'Unknown Album'
              }}
              category="overlay"
              currentValue={notPlayingSettings.not_playing_album}
              onUpdate={(key, val) => updateNotPlayingSetting(key, val)}
              onTriggerAction={() => {}}
            />
          </div>
        </div>
      );
    }

    return (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
            {plugins
            .filter((p) => p.category === 'overlay' && p.id === activeOverlay)
            .map((plugin) => (
                <div key={plugin.id} className="section" style={{ margin: 0 }}>
                <h2 className="section-title">{plugin.name} Content</h2>
                <div className="settings-card" style={{ border: 'none', background: 'transparent', padding: 0 }}>
                    {plugin.fields
                    .filter(f => (f.field_type.type === 'Text' && !f.key.toLowerCase().includes('color') && !f.key.toLowerCase().includes('colour')) || f.field_type.type === 'Action' || f.field_type.type === 'Number' || f.field_type.type === 'Boolean')
                    .map((field) => (
                    <div key={field.key} onFocus={() => {
                        if (field.field_type.type === 'Text') {
                            focusedInputRef.current = { pluginId: plugin.id, key: field.key };
                        }
                    }}>
                        <SettingsField
                            field={field}
                            category={plugin.category}
                            currentValue={pluginSettings[plugin.id]?.[field.key]}
                            onUpdate={(key, value) => updatePluginSetting(plugin.id, key, value)}
                            onTriggerAction={(actionName) => triggerAction(plugin.id, actionName)}
                        />
                    </div>
                    ))}
                </div>
                </div>
            ))}
            
            {activeOverlay === 'twitch_alerts' && (
                <div className="section" style={{ margin: 0 }}>
                    <h2 className="section-title">Available Variables (Click to Insert)</h2>
                    <TwitchAlertPlaceholderGrid onInsertPlaceholder={handleInsertAlertPlaceholder} />
                </div>
            )}
        </div>
    );
  };

  const renderPreview = () => {
    switch (activeOverlay) {
      case 'now_playing':
        return (
          <div style={{ transform: 'scale(0.75)', transformOrigin: 'center' }}>
            <OverlayPreview
              playback={playback}
              progressPercent={progressPercent}
              displayProgressMs={displayProgressMs}
              formatTime={formatTime}
            />
          </div>
        );
      case 'twitch_hype_train':
        return (
          <div style={{ transform: 'scale(0.75)', transformOrigin: 'center', width: '380px' }}>
            <HypeTrainPreview 
              state={hypeTrainState} 
              settings={pluginSettings['twitch_hype_train'] || {}} 
            />
          </div>
        );
      case 'twitch_alerts':
        return (
          <div style={{ transform: 'scale(0.75)', transformOrigin: 'center', width: '380px' }}>
            <TwitchAlertPreview
              state={alertState}
              settings={pluginSettings['twitch_alerts'] || {}}
            />
          </div>
        );
      default:
        return (
          <div style={{ color: 'var(--text-secondary)', fontSize: '14px' }}>
            Preview for {activeOverlay} pending…
          </div>
        );
    }
  };

  return (
    <div className="view-pane" style={{ flexDirection: 'column', gap: '20px', height: '100%', padding: 0 }}>
      {/* Overlay Tabs */}
      <div style={{ display: 'flex', gap: '4px', borderBottom: '1px solid var(--border)', paddingBottom: '0' }}>
        {overlayTabs.map((tab) => (
          <button
            key={tab.id}
            type="button"
            onClick={() => setActiveOverlay(tab.id)}
            style={{
              background: activeOverlay === tab.id ? 'var(--bg-card)' : 'transparent',
              color: activeOverlay === tab.id ? 'var(--text-main)' : 'var(--text-secondary)',
              border: '1px solid ' + (activeOverlay === tab.id ? 'var(--border)' : 'transparent'),
              borderBottom: activeOverlay === tab.id ? '2px solid var(--accent-primary)' : '1px solid transparent',
              borderRadius: '6px 6px 0 0',
              padding: '8px 16px',
              fontSize: '13px',
              fontWeight: 600,
              marginBottom: '-1px',
              cursor: 'pointer',
              transition: 'all 0.2s ease',
              boxShadow: 'none'
            }}
          >
            {tab.name}
          </button>
        ))}
      </div>

      <div style={{ display: 'flex', flex: 1, minHeight: 0, gap: '24px' }}>
        {/* Left Column: Preview (Fixed) and Text Settings (Scrollable) */}
        <div style={{ flex: '0 0 420px', display: 'flex', flexDirection: 'column', gap: '24px', paddingRight: '4px', overflow: 'hidden' }}>
          
          {/* Preview Container - Fixed at Top */}
          <div style={{ flex: '0 0 240px', display: 'flex', flexDirection: 'column' }}>
            <h1 className="view-title" style={{ marginBottom: '12px', fontSize: '16px', opacity: 0.8 }}>Live Overlay Preview</h1>
            <div className="panoptic-overlay-preview-container" style={{
              flex: 1,
              borderRadius: '8px',
              border: '1px solid var(--border)',
              background: 'radial-gradient(circle at center, rgba(139, 92, 246, 0.03) 0%, transparent 80%), #06070a',
              overflow: 'hidden',
              display: 'flex',
              justifyContent: 'center',
              alignItems: 'center',
              padding: '20px',
              position: 'relative'
            }}>
              {/* Live CSS Injection for Preview */}
              <style>{overlaysCss[activeOverlay] || ''}</style>
              {renderPreview()}
            </div>
          </div>

          {/* Configuration Sections - Scrollable */}
          <div style={{ flex: 1, overflowY: 'auto', paddingBottom: '20px' }}>
            <h1 className="view-title" style={{ marginBottom: '12px', fontSize: '16px', opacity: 0.8 }}>Overlay Settings</h1>
            {renderTextSettings()}
          </div>
        </div>

        {/* Right Column: CSS Editor */}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minHeight: 0 }}>
          <h1 className="view-title" style={{ marginBottom: '12px', fontSize: '16px', opacity: 0.8 }}>Custom CSS</h1>
          <div className="section" style={{ flex: 1, display: 'flex', flexDirection: 'column', margin: 0, minHeight: 0 }}>
            <CodeMirror
              value={overlaysCss[activeOverlay] || ''}
              height="100%"
              extensions={[
                css(),
                customCompletionExtension,
                cssSyntaxLinter,
                lintGutter()
              ]}
              theme="dark"
              style={{
                flex: 1,
                width: '100%',
                fontSize: '12.5px',
                fontFamily: "'JetBrains Mono', monospace",
                minHeight: '300px',
                borderRadius: '6px',
                border: '1px solid var(--border)',
                overflow: 'hidden',
                textAlign: 'left'
              }}
              onChange={(value) => {
                setOverlaysCss(activeOverlay, value);
              }}
            />
          </div>
        </div>
      </div>
    </div>
  );
};
