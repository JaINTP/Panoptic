import React from 'react';

export interface PlaybackState {
  title: string;
  artist: string;
  album: string;
  art_source: string;
  progress_ms: number;
  duration_ms: number;
  is_playing: boolean;
}

interface OverlayPreviewProps {
  playback: PlaybackState;
  progressPercent: number;
  displayProgressMs: number;
  formatTime: (ms: number) => string;
  settings?: {
    not_playing_title?: string;
    not_playing_artist?: string;
    not_playing_album?: string;
  };
}

export const OverlayPreview: React.FC<OverlayPreviewProps> = ({
  playback,
  progressPercent,
  displayProgressMs,
  formatTime,
  settings,
}) => {
  const artSrc = React.useMemo(() => {
    if (!playback.art_source) {
      return 'https://i.scdn.co/image/ab67616d0000b27370364408e063f2f0c76f4e17';
    }
    if (playback.art_source.startsWith('file://')) {
      const buster = Date.now();
      return `http://127.0.0.1:3000/art?v=${encodeURIComponent(playback.art_source)}&t=${buster}`;
    }
    return playback.art_source;
  }, [playback.art_source, playback.title, playback.artist]);

  return (
    <div className="panoptic-overlay-wrapper" data-playing={playback.is_playing}>
      <div className="panoptic-overlay-card">
        <div className="panoptic-overlay-art-container">
          <img
            src={artSrc}
            alt="Album Art"
            className="panoptic-overlay-album-art"
          />
        </div>
        <div className="panoptic-overlay-text-container">
          <div className="panoptic-overlay-track-title">
            {playback.title || settings?.not_playing_title || 'Not Playing'}
          </div>
          <div className="panoptic-overlay-track-artist">
            {playback.title ? (playback.artist || 'Unknown Artist') : (settings?.not_playing_artist || 'Unknown Artist')}
          </div>
          <div className="panoptic-overlay-track-album">
            {playback.title ? (playback.album || 'Unknown Album') : (settings?.not_playing_album || 'Unknown Album')}
          </div>

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
  );
};
