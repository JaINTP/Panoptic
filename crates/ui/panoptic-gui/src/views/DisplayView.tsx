import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import CodeMirror from '@uiw/react-codemirror';
import { css, cssLanguage } from '@codemirror/lang-css';
import { linter, lintGutter, Diagnostic } from '@codemirror/lint';
import { syntaxTree } from '@codemirror/language';
import { CompletionContext, CompletionResult } from '@codemirror/autocomplete';
import { OverlayPreview, PlaybackState } from '../components/OverlayPreview';
import { HypeTrainPreview, HypeTrainState } from '../components/HypeTrainPreview';
import { TwitchAlertPreview, AlertState } from '../components/TwitchAlertPreview';
import { TwitchChatPreview, ChatState, ChatMessageData } from '../components/TwitchChatPreview';
import { PomodoroPreview, PomodoroState, DEFAULT_POMODORO_STATE, POMODORO_DEFAULT_CSS } from '../components/PomodoroPreview';
import { SettingsField, PluginDef } from '../components/SettingsField';
import { TwitchAlertPlaceholderGrid } from '../components/TwitchAlertPlaceholderGrid';
import { TwitchChatPlaceholderGrid } from '../components/TwitchChatPlaceholderGrid';
import {
  StreamGoalsPreview,
  GoalConfig,
  CustomVar,
  SessionStats,
  DEFAULT_SESSION_STATS,
  STREAM_GOALS_DEFAULT_CSS,
} from '../components/StreamGoalsPreview';
import { StreamGoalsConfig } from '../components/StreamGoalsConfig';

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
  { label: '.alert-text-content', type: 'class', detail: 'Text within an alert' },
  { label: '.chat-message', type: 'class', detail: 'Individual chat message' },
  { label: '.chat-message-broadcaster', type: 'class', detail: 'Message from broadcaster' },
  { label: '.chat-message-mod', type: 'class', detail: 'Message from moderator' },
  { label: '.chat-message-vip', type: 'class', detail: 'Message from VIP' },
  { label: '.chat-message-sub', type: 'class', detail: 'Message from subscriber' },
  { label: '.chat-header', type: 'class', detail: 'Chat message header' },
  { label: '.chat-username', type: 'class', detail: 'Chatter username' },
  { label: '.chat-pronouns', type: 'class', detail: 'Chatter pronouns' },
  { label: '.chat-text', type: 'class', detail: 'Chat message body' }
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

  const [chatState, setChatState] = useState<ChatState>({
    messages: [],
  });

  const [pomodoroState, setPomodoroState] = useState<PomodoroState>(DEFAULT_POMODORO_STATE);

  // ── Stream Goals state ─────────────────────────────────────────────────────
  const [streamGoals, setStreamGoals] = useState<GoalConfig[]>([]);
  const [streamCustomVars, setStreamCustomVars] = useState<CustomVar[]>([]);
  const [sessionStats, setSessionStats] = useState<SessionStats>(DEFAULT_SESSION_STATS);

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
    const unlistenChat = listen<ChatMessageData>('twitch_chat_message', (event) => {
      setChatState(prev => {
        const newMsgs = [...prev.messages, event.payload];
        if (newMsgs.length > 50) newMsgs.shift();
        return { messages: newMsgs };
      });
    });
    const unlistenPomodoro = listen<PomodoroState>('pomodoro_tick', (event) => {
      setPomodoroState(event.payload);
    });
    // Stream Goals: live session stats
    const unlistenStats = listen<SessionStats>('session_stats_update', (event) => {
      setSessionStats(event.payload);
    });
    // Stream Goals: custom var changes (from other sources / actions)
    const unlistenVars = listen<CustomVar[]>('stream_goals_custom_var_update', (event) => {
      setStreamCustomVars(event.payload);
    });

    return () => {
      unlistenHype.then(f => f());
      unlistenHypeClear.then(f => f());
      unlistenAlert.then(f => f());
      unlistenChat.then(f => f());
      unlistenPomodoro.then(f => f());
      unlistenStats.then(f => f());
      unlistenVars.then(f => f());
    };
  }, []);

  // Load initial stream goals config
  useEffect(() => {
    invoke<{ goals: GoalConfig[]; custom_vars: CustomVar[] }>('get_stream_goals_config')
      .then((cfg) => {
        setStreamGoals(cfg.goals ?? []);
        setStreamCustomVars(cfg.custom_vars ?? []);
      })
      .catch(() => {});
    invoke<SessionStats>('get_session_stats')
      .then(setSessionStats)
      .catch(() => {});
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

  const getOverlayUrl = (id: string) => {
    const base = 'http://127.0.0.1:3000';
    switch (id) {
      case 'now_playing': return `${base}/overlay/now-playing`;
      case 'twitch_hype_train': return `${base}/overlay/twitch/hype-train`;
      case 'twitch_alerts': return `${base}/overlay/twitch/alerts`;
      case 'twitch_chat': return `${base}/overlay/twitch/chat`;
      case 'stream_goals': return `${base}/overlay/stream-goals`;
      default: return `${base}/overlay/${id}`;
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  const renderTextSettings = () => {
    const overlayUrl = getOverlayUrl(activeOverlay);
    const urlDisplay = (
      <div className="section" style={{ margin: '0 0 20px 0' }}>
        <h2 className="section-title">Browser Source URL</h2>
        <div className="settings-card" style={{ 
          display: 'flex', 
          alignItems: 'center', 
          gap: '12px', 
          background: 'rgba(0,0,0,0.3)',
          padding: '10px 14px',
          borderRadius: '6px',
          border: '1px dashed var(--border)'
        }}>
          <code style={{ 
            flex: 1, 
            fontSize: '11px', 
            color: 'var(--accent-primary)',
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap'
          }}>
            {overlayUrl}
          </code>
          <button 
            onClick={() => copyToClipboard(overlayUrl)}
            style={{
              padding: '4px 10px',
              fontSize: '10px',
              background: 'var(--bg-app)',
              border: '1px solid var(--border)',
              borderRadius: '4px',
              color: 'var(--text-main)',
              cursor: 'pointer'
            }}
          >
            Copy
          </button>
        </div>
      </div>
    );

    if (activeOverlay === 'now_playing') {
      return (
        <div style={{ display: 'flex', flexDirection: 'column' }}>
          {urlDisplay}
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
                category={null}
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
                category={null}
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
                category={null}
                currentValue={notPlayingSettings.not_playing_album}
                onUpdate={(key, val) => updateNotPlayingSetting(key, val)}
                onTriggerAction={() => {}}
              />
            </div>
          </div>
        </div>
      );
    }

    return (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
            {urlDisplay}
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

            {activeOverlay === 'twitch_chat' && (
                <div className="section" style={{ margin: 0 }}>
                    <h2 className="section-title">Available Variables (Click to Insert)</h2>
                    <TwitchChatPlaceholderGrid onInsertPlaceholder={handleInsertAlertPlaceholder} />
                </div>
            )}

            {activeOverlay === 'stream_goals' && (
                <div className="section" style={{ margin: 0 }}>
                    <StreamGoalsConfig
                        goals={streamGoals}
                        customVars={streamCustomVars}
                        stats={sessionStats}
                        onGoalsChange={setStreamGoals}
                        onCustomVarsChange={setStreamCustomVars}
                        onStatsReset={() => setSessionStats(DEFAULT_SESSION_STATS)}
                    />
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
      case 'twitch_chat':
        return (
          <div style={{ transform: 'scale(0.75)', transformOrigin: 'center', width: '400px' }}>
            <TwitchChatPreview
              state={chatState}
              settings={pluginSettings['twitch_chat'] || {}}
            />
          </div>
        );
      case 'pomodoro':
        return (
          <div style={{ transform: 'scale(0.85)', transformOrigin: 'center' }}>
            <PomodoroPreview state={pomodoroState} />
          </div>
        );
      case 'stream_goals':
        return (
          <div style={{ transform: 'scale(0.75)', transformOrigin: 'center', width: '380px' }}>
            <StreamGoalsPreview
              goals={streamGoals}
              customVars={streamCustomVars}
              stats={sessionStats}
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
      {/* Overlay Tabs & Aesthetic Pack Selector */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', borderBottom: '1px solid var(--border)', paddingBottom: '0' }}>
        <div style={{ display: 'flex', gap: '4px' }}>
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

        <div style={{ display: 'flex', alignItems: 'center', gap: '10px', paddingRight: '8px', paddingBottom: '6px' }}>
          <span style={{ fontSize: '11px', color: 'var(--text-secondary)', fontWeight: 600, letterSpacing: '0.05em', textTransform: 'uppercase' }}>Aesthetic Pack:</span>
          <select
            onChange={async (e) => {
              const packId = e.target.value;
              if (packId) {
                try {
                  await invoke('apply_aesthetic_pack', { packId });
                } catch (err) {
                  console.error('Failed to apply aesthetic pack:', err);
                  alert(`Failed to apply pack: ${err}`);
                }
              }
            }}
            defaultValue=""
            style={{
              background: 'var(--bg-card)',
              color: 'var(--text-main)',
              border: '1px solid var(--border)',
              borderRadius: '6px',
              padding: '6px 12px',
              fontSize: '12.5px',
              fontWeight: 600,
              cursor: 'pointer',
              outline: 'none',
              transition: 'border-color 0.2s, box-shadow 0.2s',
              boxShadow: 'var(--shadow-sm)',
            }}
            onFocus={(e) => {
              e.target.style.borderColor = 'var(--border-focus)';
              e.target.style.boxShadow = '0 0 0 2px var(--accent-primary-glow)';
            }}
            onBlur={(e) => {
              e.target.style.borderColor = 'var(--border)';
              e.target.style.boxShadow = 'var(--shadow-sm)';
            }}
          >
            <option value="" disabled>Select a pack...</option>
            <option value="cyber">Cyberpunk Neon</option>
            <option value="eldritch">Eldritch Tomes</option>
            <option value="rpg90s">RPG Retro 90s</option>
            <option value="salem">Salem Witch Cauldron</option>
          </select>
        </div>
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
              {/* Live CSS Injection for Preview - defaults first, custom rules win */}
              <style>{activeOverlay === 'pomodoro' ? POMODORO_DEFAULT_CSS : ''}{activeOverlay === 'stream_goals' ? STREAM_GOALS_DEFAULT_CSS : ''}{overlaysCss[activeOverlay] || ''}</style>
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
