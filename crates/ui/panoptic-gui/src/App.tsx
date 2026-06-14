import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { openUrl } from '@tauri-apps/plugin-opener';
import CodeMirror from '@uiw/react-codemirror';
import { css, cssLanguage } from '@codemirror/lang-css';
import { linter, lintGutter, Diagnostic } from '@codemirror/lint';
import { syntaxTree } from '@codemirror/language';
import { CompletionContext, CompletionResult } from '@codemirror/autocomplete';
import { 
  HardDrive, 
  ShieldCheck, 
  Type, 
  Monitor
} from 'lucide-react';
import './index.css';
import './overlay.css';

type View = 'storage' | 'auth' | 'output' | 'display';

interface PlaybackState {
  title: string;
  artist: string;
  album: string;
  art_source: string;
  progress_ms: number;
  duration_ms: number;
  is_playing: boolean;
}

const formatTime = (ms: number) => {
  if (isNaN(ms) || ms <= 0) return '0:00';
  const totalSecs = Math.floor(ms / 1000);
  const mins = Math.floor(totalSecs / 60);
  const secs = totalSecs % 60;
  return `${mins}:${secs < 10 ? '0' : ''}${secs}`;
};

const PANOPTIC_CLASSES = [
  { label: '.panoptic-overlay-wrapper', type: 'class', detail: 'Overlay container wrapper' },
  { label: '.panoptic-overlay-card', type: 'class', detail: 'The main display card / vinyl disc' },
  { label: '.panoptic-overlay-art-container', type: 'class', detail: 'Art container wrapper' },
  { label: '.panoptic-overlay-album-art', type: 'class', detail: 'Album artwork image element' },
  { label: '.panoptic-overlay-text-container', type: 'class', detail: 'Info text overlay container' },
  { label: '.panoptic-overlay-track-title', type: 'class', detail: 'Track title text' },
  { label: '.panoptic-overlay-track-artist', type: 'class', detail: 'Track artist text' },
  { label: '.panoptic-overlay-track-album', type: 'class', detail: 'Track album name text' },
  { label: '.panoptic-overlay-progress-section', type: 'class', detail: 'Progress bar section wrapper' },
  { label: '.panoptic-overlay-progress-container', type: 'class', detail: 'Progress bar background track' },
  { label: '.panoptic-overlay-progress-bar', type: 'class', detail: 'Active progress bar fill' },
  { label: '.panoptic-overlay-time-display', type: 'class', detail: 'Time elapsed display container' },
  { label: '.panoptic-overlay-time-separator', type: 'class', detail: 'Time separator element' }
];

const PANOPTIC_VARS = [
  // General layout properties
  { label: '--panoptic-overlay-card-display', type: 'variable', detail: 'flex | block | grid' },
  { label: '--panoptic-overlay-card-gap', type: 'variable', detail: 'Gap size (e.g. 20px)' },
  { label: '--panoptic-overlay-card-background', type: 'variable', detail: 'Card bg (e.g. rgba(19, 21, 28, 0.75))' },
  { label: '--panoptic-overlay-card-border-style', type: 'variable', detail: 'solid | none | dashed' },
  { label: '--panoptic-overlay-card-border-width', type: 'variable', detail: 'Border width (e.g. 1px)' },
  { label: '--panoptic-overlay-card-border-color', type: 'variable', detail: 'Border color' },
  { label: '--panoptic-overlay-card-backdrop-blur', type: 'variable', detail: 'Backdrop blur (e.g. 16px)' },
  { label: '--panoptic-overlay-card-padding', type: 'variable', detail: 'Card padding' },
  { label: '--panoptic-overlay-card-border-radius', type: 'variable', detail: 'Corner radius (e.g. 16px)' },
  { label: '--panoptic-overlay-card-width', type: 'variable', detail: 'Card width (e.g. 380px)' },
  { label: '--panoptic-overlay-card-box-shadow', type: 'variable', detail: 'Card box shadow' },

  // Album Art variables
  { label: '--panoptic-overlay-album-art-width', type: 'variable', detail: 'Art width' },
  { label: '--panoptic-overlay-album-art-height', type: 'variable', detail: 'Art height' },
  { label: '--panoptic-overlay-album-art-border-radius', type: 'variable', detail: 'Corner radius' },
  { label: '--panoptic-overlay-album-art-object-fit', type: 'variable', detail: 'cover | contain | fill' },
  { label: '--panoptic-overlay-album-art-box-shadow', type: 'variable', detail: 'Art box shadow' },

  // Typography - Title
  { label: '--panoptic-overlay-track-title-font-family', type: 'variable', detail: 'Font family' },
  { label: '--panoptic-overlay-track-title-font-size', type: 'variable', detail: 'Font size' },
  { label: '--panoptic-overlay-track-title-font-weight', type: 'variable', detail: 'Font weight' },
  { label: '--panoptic-overlay-track-title-text-color', type: 'variable', detail: 'Title color' },

  // Typography - Artist
  { label: '--panoptic-overlay-track-artist-font-family', type: 'variable', detail: 'Font family' },
  { label: '--panoptic-overlay-track-artist-font-size', type: 'variable', detail: 'Font size' },
  { label: '--panoptic-overlay-track-artist-font-weight', type: 'variable', detail: 'Font weight' },
  { label: '--panoptic-overlay-track-artist-text-color', type: 'variable', detail: 'Artist color' },

  // Typography - Album
  { label: '--panoptic-overlay-track-album-font-family', type: 'variable', detail: 'Font family' },
  { label: '--panoptic-overlay-track-album-font-size', type: 'variable', detail: 'Font size' },
  { label: '--panoptic-overlay-track-album-font-weight', type: 'variable', detail: 'Font weight' },
  { label: '--panoptic-overlay-track-album-text-color', type: 'variable', detail: 'Album color' },
  { label: '--panoptic-overlay-track-album-letter-spacing', type: 'variable', detail: 'Letter spacing' },

  // Progress Bar
  { label: '--panoptic-overlay-progress-bar-height', type: 'variable', detail: 'Bar height' },
  { label: '--panoptic-overlay-progress-bar-background', type: 'variable', detail: 'Track color' },
  { label: '--panoptic-overlay-progress-bar-border-radius', type: 'variable', detail: 'Corner radius' },
  { label: '--panoptic-overlay-progress-bar-fill-gradient', type: 'variable', detail: 'Fill color/gradient' },

  // Time Display
  { label: '--panoptic-overlay-time-display-font-family', type: 'variable', detail: 'Font family' },
  { label: '--panoptic-overlay-time-display-font-size', type: 'variable', detail: 'Font size' },
  { label: '--panoptic-overlay-time-display-text-color', type: 'variable', detail: 'Time color' },

  // Vinyl Theme specific
  { label: '--vinyl-size', type: 'variable', detail: 'Spinning vinyl size (e.g. 320px)' },
  { label: '--ring-width', type: 'variable', detail: 'Outer progress ring width (e.g. 4px)' },
  { label: '--accent-glow', type: 'variable', detail: 'Glow drop-shadow color' },
  { label: '--overlay-bg', type: 'variable', detail: 'Text background radial gradient center' }
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

function App() {
  const [activeView, setActiveView] = useState<View>('display');
  const [spotifyAuth, setSpotifyAuth] = useState(false);
  const [clientId, setClientId] = useState('');
  const [template, setTemplate] = useState('Now Playing: {title} by {artist}');
  const [cssCode, setCssCode] = useState(`/* Custom Live Overlay CSS Configurator */
:root {
  /* Card Layout & Background */
  --panoptic-overlay-card-display: flex;
  --panoptic-overlay-card-gap: 20px;
  --panoptic-overlay-card-background: rgba(19, 21, 28, 0.75);
  --panoptic-overlay-card-border-style: solid;
  --panoptic-overlay-card-border-width: 1px;
  --panoptic-overlay-card-border-color: rgba(255, 255, 255, 0.08);
  --panoptic-overlay-card-backdrop-blur: 16px;
  --panoptic-overlay-card-padding: 20px;
  --panoptic-overlay-card-border-radius: 16px;
  --panoptic-overlay-card-width: 380px;
  --panoptic-overlay-card-box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
  
  /* Album Art Image */
  --panoptic-overlay-album-art-width: 90px;
  --panoptic-overlay-album-art-height: 90px;
  --panoptic-overlay-album-art-border-radius: 10px;
  --panoptic-overlay-album-art-object-fit: cover;
  --panoptic-overlay-album-art-box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  
  /* Song Title Typography */
  --panoptic-overlay-track-title-font-family: 'Outfit', sans-serif;
  --panoptic-overlay-track-title-font-size: 17px;
  --panoptic-overlay-track-title-font-weight: 700;
  --panoptic-overlay-track-title-text-color: #ffffff;
  
  /* Artist Name Typography */
  --panoptic-overlay-track-artist-font-size: 13.5px;
  --panoptic-overlay-track-artist-font-weight: 600;
  --panoptic-overlay-track-artist-text-color: var(--accent-primary);
  
  /* Album Name Typography */
  --panoptic-overlay-track-album-font-size: 11px;
  --panoptic-overlay-track-album-text-color: var(--text-muted);
  --panoptic-overlay-track-album-letter-spacing: 0.05em;
  
  /* Progress Bar Styling */
  --panoptic-overlay-progress-bar-height: 5px;
  --panoptic-overlay-progress-bar-background: rgba(255, 255, 255, 0.08);
  --panoptic-overlay-progress-bar-border-radius: 3px;
  --panoptic-overlay-progress-bar-fill-gradient: linear-gradient(90deg, var(--accent-primary) 0%, #a78bfa 100%);
  
  /* Time Display Typography */
  --panoptic-overlay-time-display-font-family: 'JetBrains Mono', monospace;
  --panoptic-overlay-time-display-font-size: 10.5px;
  --panoptic-overlay-time-display-text-color: var(--text-secondary);
}

.panoptic-overlay-wrapper {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 100%;
  height: 100%;
}

.panoptic-overlay-card {
  display: var(--panoptic-overlay-card-display);
  gap: var(--panoptic-overlay-card-gap);
  background: var(--panoptic-overlay-card-background);
  border: var(--panoptic-overlay-card-border-width) var(--panoptic-overlay-card-border-style) var(--panoptic-overlay-card-border-color);
  backdrop-filter: blur(var(--panoptic-overlay-card-backdrop-blur));
  padding: var(--panoptic-overlay-card-padding);
  border-radius: var(--panoptic-overlay-card-border-radius);
  width: var(--panoptic-overlay-card-width);
  box-shadow: var(--panoptic-overlay-card-box-shadow);
  transition: all 0.3s ease;
}

.panoptic-overlay-album-art {
  width: var(--panoptic-overlay-album-art-width);
  height: var(--panoptic-overlay-album-art-height);
  border-radius: var(--panoptic-overlay-album-art-border-radius);
  object-fit: var(--panoptic-overlay-album-art-object-fit);
  box-shadow: var(--panoptic-overlay-album-art-box-shadow);
}

.panoptic-overlay-text-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  overflow: hidden;
}

.panoptic-overlay-track-title {
  font-family: var(--panoptic-overlay-track-title-font-family);
  font-size: var(--panoptic-overlay-track-title-font-size);
  font-weight: var(--panoptic-overlay-track-title-font-weight);
  color: var(--panoptic-overlay-track-title-text-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 2px;
}

.panoptic-overlay-track-artist {
  font-size: var(--panoptic-overlay-track-artist-font-size);
  font-weight: var(--panoptic-overlay-track-artist-font-weight);
  color: var(--panoptic-overlay-track-artist-text-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 2px;
}

.panoptic-overlay-track-album {
  font-size: var(--panoptic-overlay-track-album-font-size);
  color: var(--panoptic-overlay-track-album-text-color);
  letter-spacing: var(--panoptic-overlay-track-album-letter-spacing);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 12px;
  text-transform: uppercase;
}

.panoptic-overlay-progress-container {
  width: 100%;
  height: var(--panoptic-overlay-progress-bar-height);
  background: var(--panoptic-overlay-progress-bar-background);
  border-radius: var(--panoptic-overlay-progress-bar-border-radius);
  overflow: hidden;
  margin-bottom: 6px;
}

.panoptic-overlay-progress-bar {
  height: 100%;
  background: var(--panoptic-overlay-progress-bar-fill-gradient);
  border-radius: var(--panoptic-overlay-progress-bar-border-radius);
}

.panoptic-overlay-time-display {
  display: flex;
  justify-content: space-between;
  font-family: var(--panoptic-overlay-time-display-font-family);
  font-size: var(--panoptic-overlay-time-display-font-size);
  color: var(--panoptic-overlay-time-display-text-color);
}`);
  
  // Mock Playback State for Demonstration
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

  // Load initial settings on mount
  useEffect(() => {
    const loadSettings = async () => {
      try {
        const id = await invoke<string>('get_spotify_client_id');
        setClientId(id);
        const status = await invoke<boolean>('get_spotify_status');
        setSpotifyAuth(status);
        const savedTemplate = await invoke<string | null>('get_output_template');
        if (savedTemplate !== null) {
          setTemplate(savedTemplate);
        }
        const savedCss = await invoke<string | null>('get_overlay_css');
        if (savedCss !== null) {
          setCssCode(savedCss);
        }
        const updateStatus = await invoke<any>('get_update_status');
        if (updateStatus) {
          setUpdateVersion(updateStatus.tag_name);
          setUpdateUrl(updateStatus.html_url);
        }
      } catch (e) {
        console.error('Failed to load settings:', e);
      }
    };
    loadSettings();
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

  // Listen for native authentication success
  useEffect(() => {
    const unlisten = listen('auth_success', () => {
      setSpotifyAuth(true);
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

  // High-fidelity progress interpolation simulator
  useEffect(() => {
    if (!playback.is_playing) {
      setDisplayProgressMs(playback.progress_ms);
      return;
    }
    const interval = setInterval(() => {
      const elapsed = Date.now() - lastUpdated;
      const current = Math.min(playback.progress_ms + elapsed, playback.duration_ms);
      setDisplayProgressMs(current);
    }, 30); // ~33fps for smooth animation
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

  const handleSaveClientId = async () => {
    try {
      await invoke('set_spotify_client_id', { clientId });
      alert('Spotify Client ID saved successfully!');
    } catch (e) {
      console.error('Failed to save Client ID:', e);
      alert('Failed to save Client ID: ' + e);
    }
  };

  const handleLinkSpotify = async () => {
    if (spotifyAuth) {
      try {
        await invoke('unlink_spotify');
        setSpotifyAuth(false);
      } catch (e) {
        console.error('Failed to unlink Spotify:', e);
      }
      return;
    }
    
    try {
      await invoke('link_spotify');
      // We do NOT set spotifyAuth(true) here. 
      // We wait for the 'auth_success' event from the rust callback server.
    } catch (e) {
      console.error('Failed to link Spotify:', e);
    }
  };

  const renderView = () => {
    switch (activeView) {
      case 'display':
        return (
          <div className="view-pane" style={{ flexDirection: 'column', gap: '20px', height: '100%', padding: 0 }}>
            {/* Top Preview Pane */}
            <div style={{ flex: '0 0 240px', display: 'flex', flexDirection: 'column' }}>
              <h1 className="view-title" style={{ marginBottom: '8px', fontSize: '18px' }}>Live Overlay Preview</h1>
              <div className="panoptic-overlay-preview-container" style={{ 
                flex: 1, 
                borderRadius: '8px', 
                border: '1px solid var(--border)', 
                background: 'radial-gradient(circle at center, rgba(139, 92, 246, 0.03) 0%, transparent 80%), #06070a', 
                overflow: 'hidden', 
                display: 'flex', 
                justifyContent: 'center', 
                alignItems: 'center' 
              }}>
                <div className="panoptic-overlay-wrapper" data-playing={playback.is_playing}>
                  <div className="panoptic-overlay-card">
                    <div className="panoptic-overlay-art-container">
                      <img 
                        src={playback.art_source || 'https://i.scdn.co/image/ab67616d0000b27370364408e063f2f0c76f4e17'} 
                        alt="Album Art" 
                        className="panoptic-overlay-album-art" 
                      />
                    </div>
                    <div className="panoptic-overlay-text-container">
                      <div className="panoptic-overlay-track-title">{playback.title || 'Not Playing'}</div>
                      <div className="panoptic-overlay-track-artist">{playback.artist || 'Unknown Artist'}</div>
                      <div className="panoptic-overlay-track-album">{playback.album || 'Unknown Album'}</div>
                      
                      <div className="panoptic-overlay-progress-section">
                        <div className="panoptic-overlay-progress-container">
                          <div 
                            className="panoptic-overlay-progress-bar" 
                            style={{ width: `${progressPercent}%` }}
                          ></div>
                        </div>
                        <div className="panoptic-overlay-time-display">
                          <span className="panoptic-overlay-time-current">{formatTime(displayProgressMs)}</span>
                          <span className="panoptic-overlay-time-separator">/</span>
                          <span className="panoptic-overlay-time-duration">{formatTime(playback.duration_ms)}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            {/* Bottom Editor Pane */}
            <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minHeight: 0 }}>
              <h1 className="view-title" style={{ marginBottom: '8px', fontSize: '18px' }}>CSS Stylesheet</h1>
              <div className="section" style={{ flex: 1, display: 'flex', flexDirection: 'column', margin: 0, minHeight: 0 }}>
                <CodeMirror
                  value={cssCode}
                  height="100%"
                  extensions={[
                    css(),
                    customCompletionExtension,
                    cssSyntaxLinter,
                    lintGutter()
                  ]}
                  onChange={(value) => {
                    setCssCode(value);
                    invoke('set_overlay_css', { css: value }).catch(err => 
                      console.error('Failed to save CSS:', err)
                    );
                  }}
                  theme="dark"
                  style={{ 
                    flex: 1, 
                    width: '100%',
                    fontSize: '12.5px', 
                    fontFamily: "'JetBrains Mono', monospace",
                    minHeight: '150px',
                    borderRadius: '6px',
                    border: '1px solid var(--border)',
                    overflow: 'hidden',
                    textAlign: 'left'
                  }}
                />
              </div>
            </div>
          </div>
        );
      case 'storage':
        return (
          <div className="view-pane view-pane-scrollable">
            <h1 className="view-title">Storage & Environment</h1>
            <div className="section">
              <h2 className="section-title">System Paths</h2>
              <div className="row">
                <div className="label">Artwork Cache</div>
                <div className="input-group">
                  <input type="text" readOnly value="/home/user/.cache/panoptic/artworks" />
                  <button className="btn-secondary">Browse...</button>
                </div>
              </div>
              <div className="row">
                <div className="label">OpenAPI Spec Store</div>
                <div className="input-group">
                  <input type="text" readOnly value="/home/user/.config/panoptic/schema" />
                  <button className="btn-secondary">Browse...</button>
                </div>
              </div>
            </div>
          </div>
        );
      case 'auth':
        return (
          <div className="view-pane view-pane-scrollable">
            <h1 className="view-title">Authentication</h1>
            
            <div className="section">
              <h2 className="section-title">Developer App Settings</h2>
              <div className="settings-card">
                <p style={{ fontSize: '13px', color: 'var(--text-secondary)', marginBottom: '16px', lineHeight: '1.5' }}>
                  By default, Panoptic uses a built-in shared Spotify application. If you prefer to use your own quota or experience rate limits, you can register your own application in the <strong>Spotify Developer Dashboard</strong> and provide its Client ID here.
                </p>
                <div className="row">
                  <div className="label">Custom Client ID</div>
                  <div className="input-group">
                    <input 
                      type="text" 
                      placeholder="e.g. 299d6d15655c4d3da481f964a2754d92"
                      value={clientId}
                      onChange={(e) => setClientId(e.target.value)}
                    />
                    <button className="btn-primary" onClick={handleSaveClientId}>
                      Save ID
                    </button>
                  </div>
                  <span style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
                    Make sure to set your Spotify Developer app's Redirect URI to: <code>http://127.0.0.1:3000/callback</code>
                  </span>
                </div>
              </div>
            </div>

            <div className="section">
              <h2 className="section-title">Spotify Integration</h2>
              <div className="auth-card">
                <div>
                  <div className="label">Spotify Ingestion</div>
                  <span className={`badge ${spotifyAuth ? 'badge-emerald' : 'badge-amber'}`}>
                    {spotifyAuth ? 'Connected' : 'Local Pipes Active'}
                  </span>
                </div>
                <button 
                  className={`btn-${spotifyAuth ? 'secondary' : 'primary'}`}
                  onClick={handleLinkSpotify}
                >
                  {spotifyAuth ? 'Unlink Spotify' : 'Link Spotify'}
                </button>
              </div>
            </div>
          </div>
        );
      case 'output': {
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
            invoke('set_output_template', { template: newVal }).catch(err => 
              console.error('Failed to save template:', err)
            );
            setTimeout(() => {
              textarea.focus();
              textarea.selectionStart = textarea.selectionEnd = start + placeholder.length;
            }, 0);
          } else {
            const newVal = template + placeholder;
            setTemplate(newVal);
            invoke('set_output_template', { template: newVal }).catch(err => 
              console.error('Failed to save template:', err)
            );
          }
        };

        const renderPlaceholderItem = (code: string, label: string) => {
          return (
            <div 
              className="placeholder-item" 
              onClick={() => insertPlaceholder(code)}
              title="Click to insert at cursor"
            >
              <code style={{ color: 'var(--accent-primary-hover)', fontFamily: 'monospace', fontSize: '12.5px', fontWeight: '600' }}>{code}</code>
              <span style={{ fontSize: '11px', color: 'var(--text-secondary)' }}>{label}</span>
            </div>
          );
        };

        return (
          <div className="view-pane view-pane-scrollable">
            <h1 className="view-title">Output Templating</h1>
            <div className="section" style={{ marginBottom: '16px' }}>
              <h2 className="section-title">Template String</h2>
              <textarea 
                className="code-editor" 
                style={{ height: '80px', marginBottom: '8px' }}
                value={template}
                onChange={(e) => {
                  const val = e.target.value;
                  setTemplate(val);
                  invoke('set_output_template', { template: val }).catch(err => 
                    console.error('Failed to save template:', err)
                  );
                }}
              />
            </div>
            <div className="section" style={{ marginBottom: '16px' }}>
              <h2 className="section-title">Available Placeholders (Click to Insert)</h2>
              
              <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
                <div>
                  <h3 style={{ fontSize: '11px', color: 'var(--text-muted)', fontWeight: '700', marginBottom: '8px', textTransform: 'uppercase', letterSpacing: '0.05em' }}>Metadata</h3>
                  <div style={{ 
                    display: 'grid', 
                    gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))', 
                    gap: '12px', 
                    padding: '12px 16px', 
                    borderRadius: '6px', 
                    border: '1px solid var(--border)', 
                    backgroundColor: 'rgba(0, 0, 0, 0.15)' 
                  }}>
                    {renderPlaceholderItem('{title}', 'Track Title')}
                    {renderPlaceholderItem('{artist}', 'Artist Name(s)')}
                    {renderPlaceholderItem('{album}', 'Album Name')}
                  </div>
                </div>

                <div>
                  <h3 style={{ fontSize: '11px', color: 'var(--text-muted)', fontWeight: '700', marginBottom: '8px', textTransform: 'uppercase', letterSpacing: '0.05em' }}>Formatted Time (Recommended)</h3>
                  <div style={{ 
                    display: 'grid', 
                    gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))', 
                    gap: '12px', 
                    padding: '12px 16px', 
                    borderRadius: '6px', 
                    border: '1px solid var(--border)', 
                    backgroundColor: 'rgba(0, 0, 0, 0.15)' 
                  }}>
                    {renderPlaceholderItem('{progress}', 'Smart Progress (e.g. 3:04)')}
                    {renderPlaceholderItem('{duration}', 'Smart Duration (e.g. 4:12)')}
                  </div>
                </div>

                <div>
                  <h3 style={{ fontSize: '11px', color: 'var(--text-muted)', fontWeight: '700', marginBottom: '8px', textTransform: 'uppercase', letterSpacing: '0.05em' }}>Detailed Time (Progress)</h3>
                  <div style={{ 
                    display: 'grid', 
                    gridTemplateColumns: 'repeat(auto-fit, minmax(160px, 1fr))', 
                    gap: '12px', 
                    padding: '12px 16px', 
                    borderRadius: '6px', 
                    border: '1px solid var(--border)', 
                    backgroundColor: 'rgba(0, 0, 0, 0.15)' 
                  }}>
                    {renderPlaceholderItem('{progress_h}', 'Hours (unpadded)')}
                    {renderPlaceholderItem('{progress_m}', 'Mins of hour (padded)')}
                    {renderPlaceholderItem('{progress_s}', 'Secs of min (padded)')}
                    {renderPlaceholderItem('{progress_m_raw}', 'Mins of hour (unpadded)')}
                    {renderPlaceholderItem('{progress_s_raw}', 'Secs of min (unpadded)')}
                    {renderPlaceholderItem('{progress_m_total}', 'Total minutes (unpadded)')}
                    {renderPlaceholderItem('{progress_m_total_padded}', 'Total minutes (padded)')}
                    {renderPlaceholderItem('{progress_s_total}', 'Total seconds (unpadded)')}
                    {renderPlaceholderItem('{progress_ms}', 'Progress in milliseconds')}
                  </div>
                </div>

                <div>
                  <h3 style={{ fontSize: '11px', color: 'var(--text-muted)', fontWeight: '700', marginBottom: '8px', textTransform: 'uppercase', letterSpacing: '0.05em' }}>Detailed Time (Duration)</h3>
                  <div style={{ 
                    display: 'grid', 
                    gridTemplateColumns: 'repeat(auto-fit, minmax(160px, 1fr))', 
                    gap: '12px', 
                    padding: '12px 16px', 
                    borderRadius: '6px', 
                    border: '1px solid var(--border)', 
                    backgroundColor: 'rgba(0, 0, 0, 0.15)' 
                  }}>
                    {renderPlaceholderItem('{duration_h}', 'Hours (unpadded)')}
                    {renderPlaceholderItem('{duration_m}', 'Mins of hour (padded)')}
                    {renderPlaceholderItem('{duration_s}', 'Secs of min (padded)')}
                    {renderPlaceholderItem('{duration_m_raw}', 'Mins of hour (unpadded)')}
                    {renderPlaceholderItem('{duration_s_raw}', 'Secs of min (unpadded)')}
                    {renderPlaceholderItem('{duration_m_total}', 'Total minutes (unpadded)')}
                    {renderPlaceholderItem('{duration_m_total_padded}', 'Total minutes (padded)')}
                    {renderPlaceholderItem('{duration_s_total}', 'Total seconds (unpadded)')}
                    {renderPlaceholderItem('{duration_ms}', 'Duration in milliseconds')}
                  </div>
                </div>
              </div>
            </div>
            <div className="section" style={{ marginBottom: '16px' }}>
              <h2 className="section-title">Resulting Message Preview</h2>
              <div style={{
                padding: '16px',
                borderRadius: '8px',
                border: '1px solid var(--border)',
                backgroundColor: 'var(--bg-card)',
                fontFamily: 'monospace',
                fontSize: '14px',
                color: 'var(--text-main)',
                minHeight: '50px',
                display: 'flex',
                alignItems: 'center'
              }}>
                {formattedOutput || <span style={{ color: 'var(--text-muted)' }}>(Empty template output)</span>}
              </div>
            </div>
          </div>
        );
      }

    }
  };

  return (
    <div className="app-container">
      <style id="panoptic-live-custom-css">{cssCode}</style>
      <nav className="sidebar">
        <div className="sidebar-title">PANOPTIC v0.1.3</div>
        <a href="#" className={`sidebar-item ${activeView === 'display' ? 'active' : ''}`} onClick={() => setActiveView('display')}>
          <Monitor size={18} /> Live Overlay
        </a>
        <a href="#" className={`sidebar-item ${activeView === 'storage' ? 'active' : ''}`} onClick={() => setActiveView('storage')}>
          <HardDrive size={18} /> Storage
        </a>
        <a href="#" className={`sidebar-item ${activeView === 'auth' ? 'active' : ''}`} onClick={() => setActiveView('auth')}>
          <ShieldCheck size={18} /> Auth
        </a>
        <a href="#" className={`sidebar-item ${activeView === 'output' ? 'active' : ''}`} onClick={() => setActiveView('output')}>
          <Type size={18} /> Output
        </a>
        {updateVersion && (
          <div 
            onClick={handleOpenUpdate}
            style={{
              margin: 'auto 14px 14px 14px',
              padding: '10px 12px',
              borderRadius: '8px',
              background: 'linear-gradient(135deg, rgba(139, 92, 246, 0.15) 0%, rgba(167, 139, 250, 0.08) 100%)',
              border: '1px solid rgba(139, 92, 246, 0.35)',
              cursor: 'pointer',
              display: 'flex',
              flexDirection: 'column',
              gap: '4px',
              boxShadow: '0 4px 12px rgba(0, 0, 0, 0.25)',
              transition: 'all 0.2s ease',
              textAlign: 'left'
            }}
          >
            <span style={{ fontSize: '10px', fontWeight: '700', color: 'var(--accent-primary)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
              Update Available
            </span>
            <span style={{ fontSize: '12.5px', fontWeight: '600', color: '#ffffff' }}>
              Version {updateVersion}
            </span>
            <span style={{ fontSize: '10.5px', color: 'var(--text-secondary)' }}>
              Click to view release
            </span>
          </div>
        )}
      </nav>
      <main className="content">
        {renderView()}
      </main>
    </div>
  );
}

export default App;
