import React from 'react';

export interface HypeTrainState {
  active: boolean;
  level: number;
  total: number;
  progress: number;
  goal: number;
  top_contributions: Array<{
    user_id: string;
    user_login: string;
    user_name: string;
    type_field: string;
    total: number;
  }>;
  last_contribution?: {
    user_id: string;
    user_login: string;
    user_name: string;
    type_field: string;
    total: number;
  };
  started_at: string;
  expires_at: string;
}

interface HypeTrainPreviewProps {
  state: HypeTrainState;
  settings: Record<string, any>;
}

export const HypeTrainPreview: React.FC<HypeTrainPreviewProps> = ({ state, settings }) => {
  const progressPercent = state.goal > 0 ? Math.min(100, Math.round((state.progress / state.goal) * 100)) : 0;
  
  const ROMAN = ['I','II','III','IV','V','VI','VII','VIII','IX','X'];
  const toRoman = (n: number) => ROMAN[n - 1] || String(n);

  return (
    <div className="panoptic-overlay-wrapper twitch-notifications-preview">
      <div className="hype-train-card">
        {/* Corner Ornaments */}
        <div className="corner tl">✦</div>
        <div className="corner tr">✦</div>
        <div className="corner bl">✦</div>
        <div className="corner br">✦</div>

        {!state.active ? (
          <div className="hype-idle-state">
            <div className="status-icon-wrap">
              <div className="status-icon">💤</div>
            </div>
            <div className="hype-text">
              <div className="hype-title">{settings.inactive_title || "Hype Train"}</div>
              <div className="hype-sub">{settings.inactive_subtitle || "awaiting event…"}</div>
            </div>
          </div>
        ) : (
          <div className="hype-active-state">
            <div className="hype-header">
              <div className="hype-title-row">
                <span className="hype-icon">⚡</span>
                <span className="hype-active-title">{settings.active_title || "Hype Train Active!"}</span>
              </div>
              <div className="hype-level-badge">
                {settings.level_prefix || "Level"} {toRoman(state.level)}
              </div>
            </div>

            <div className="hype-progress-section">
              <div className="hype-progress-labels">
                <span>{settings.progress_prefix || "Progress to Level"} {toRoman(state.level + 1)}</span>
                <span className="hype-progress-pct">{progressPercent}%</span>
              </div>
              <div className="hype-progress-track">
                <div 
                  className="hype-progress-fill" 
                  style={{ width: `${progressPercent}%` }}
                ></div>
              </div>
            </div>

            <div className="hype-leaderboard-section">
              <div className="hype-leaderboard-label">{settings.leaderboard_title || "⁕ Top Contributors ⁕"}</div>
              <div className="hype-leaderboard-list">
                {state.top_contributions.length > 0 ? (
                  state.top_contributions.slice(0, 3).map((c, i) => (
                    <div key={c.user_id} className="hype-leaderboard-item" style={{ animationDelay: `${i * 0.1}s` }}>
                      <span className="hype-rank">
                        {c.type_field === 'SUBSCRIPTION' ? (
                          <span title="Subscriber" style={{ fontSize: '14px' }}>💖</span>
                        ) : (
                          <span title="Bits" style={{ fontSize: '14px' }}>💎</span>
                        )}
                      </span>
                      <span className="hype-name">{c.user_name}</span>
                      <span className="hype-amount">
                        {c.type_field === 'SUBSCRIPTION' ? `${c.total} sub${c.total !== 1 ? 's' : ''}` : `${c.total.toLocaleString()}`}
                      </span>
                    </div>
                  ))
                ) : (
                  <div style={{ color: 'var(--ht-color-subtext)', fontSize: '0.85em', fontStyle: 'italic', textAlign: 'center' }}>
                    {settings.empty_leaderboard_text || "No contributors yet…"}
                  </div>
                )}
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
