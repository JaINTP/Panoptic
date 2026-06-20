import React from 'react';

// ─── Types mirroring the Rust side ──────────────────────────────────────────

export interface GoalConfig {
  id: string;
  label: string;
  variable: string;
  target: number;
  color: string;
  show_percentage: boolean;
  show_numbers: boolean;
  milestone_celebration: boolean;
  enabled: boolean;
  steps?: number[];
}

export interface CustomVar {
  name: string;
  value: number;
  step: number;
}

export interface SessionStats {
  followers: number;
  subscribers: number;
  bits: number;
  raids: number;
  hosts: number;
  gift_subs: number;
  chat_messages: number;
  unique_chatters: number;
  new_chatters: number;
  hype_train_level: number;
  cheers_count: number;
  redemptions: number;
  viewer_count: number;
  stream_title: string;
  category: string;
}

export const DEFAULT_SESSION_STATS: SessionStats = {
  followers: 0, subscribers: 0, bits: 0, raids: 0, hosts: 0,
  gift_subs: 0, chat_messages: 0, unique_chatters: 0, new_chatters: 0,
  hype_train_level: 0, cheers_count: 0, redemptions: 0, viewer_count: 0,
  stream_title: '', category: '',
};

// ─── Helpers ─────────────────────────────────────────────────────────────────

function buildVariables(
  stats: SessionStats,
  customVars: CustomVar[],
): Record<string, number> {
  const vars: Record<string, number> = {
    followers: stats.followers,
    subscribers: stats.subscribers,
    bits: stats.bits,
    raids: stats.raids,
    hosts: stats.hosts,
    gift_subs: stats.gift_subs,
    chat_messages: stats.chat_messages,
    unique_chatters: stats.unique_chatters,
    new_chatters: stats.new_chatters,
    hype_train_level: stats.hype_train_level,
    cheers_count: stats.cheers_count,
    redemptions: stats.redemptions,
    viewer_count: stats.viewer_count,
  };
  for (const cv of customVars) {
    vars[cv.name] = cv.value;
  }
  return vars;
}

// ─── Preview component ────────────────────────────────────────────────────────

interface StreamGoalsPreviewProps {
  goals: GoalConfig[];
  customVars: CustomVar[];
  stats: SessionStats;
}

const MilestoneKeyframes = `
@keyframes sg-milestone-flash-preview {
  0%   { box-shadow: 0 0 0 rgba(var(--sg-accent-rgb-p, 157 78 221), 0); }
  25%  { box-shadow: 0 0 20px 6px rgba(var(--sg-accent-rgb-p, 157 78 221), 0.8); }
  50%  { box-shadow: 0 0 8px 2px rgba(var(--sg-accent-rgb-p, 157 78 221), 0.3); }
  75%  { box-shadow: 0 0 20px 6px rgba(var(--sg-accent-rgb-p, 157 78 221), 0.7); }
  100% { box-shadow: 0 0 14px 3px rgba(var(--sg-accent-rgb-p, 157 78 221), 0.4); }
}
@keyframes sg-fill-glow-preview {
  0%, 100% { opacity: 1; }
  50%       { opacity: 0.72; }
}
`;

export const StreamGoalsPreview: React.FC<StreamGoalsPreviewProps> = ({
  goals,
  customVars,
  stats,
}) => {
  const variables = buildVariables(stats, customVars);
  const enabled = goals.filter((g) => g.enabled !== false);

  return (
    <div className="panoptic-overlay-wrapper stream-goals-preview">
      <style>{MilestoneKeyframes}</style>
      <div
        style={{
          display: 'flex',
          flexDirection: 'column',
          gap: '10px',
          minWidth: '300px',
          maxWidth: '380px',
        }}
      >
        {enabled.length === 0 ? (
          <div
            style={{
              background: 'rgba(10,8,18,0.88)',
              border: '1px solid rgba(157,78,221,0.3)',
              borderRadius: '10px',
              padding: '16px',
              color: 'var(--text-secondary)',
              fontSize: '13px',
              textAlign: 'center',
            }}
          >
            No goals configured yet.
            <br />
            Add goals in the settings panel below.
          </div>
        ) : (
          enabled.map((goal) => {
            const current = variables[goal.variable] ?? 0;
            const steps = goal.steps && goal.steps.length > 0 ? goal.steps.map(Number) : null;
            let currentTarget = goal.target || 1;
            let currentStart = 0;
            let stepIndex = 0;
            let isMultistep = false;

            if (steps) {
              isMultistep = true;
              while (stepIndex < steps.length && current >= steps[stepIndex]) {
                stepIndex++;
              }
              if (stepIndex < steps.length) {
                currentTarget = steps[stepIndex];
                currentStart = stepIndex > 0 ? steps[stepIndex - 1] : 0;
              } else {
                currentTarget = steps[steps.length - 1];
                currentStart = steps.length > 1 ? steps[steps.length - 2] : 0;
              }
            }

            const range = currentTarget - currentStart;
            const currentProgressInStep = isMultistep ? Math.max(0, current - currentStart) : current;
            const pct = Math.min(100, Math.round((currentTarget > currentStart && range > 0) ? (currentProgressInStep / range) * 100 : (current / currentTarget) * 100));
            const isMilestone = pct >= 100 && goal.milestone_celebration !== false;
            const color = goal.color || '#9d4edd';

            // Convert hex color to rgb for milestone glow
            const hexMatch = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(color);
            const rgbStr = hexMatch
              ? `${parseInt(hexMatch[1], 16)} ${parseInt(hexMatch[2], 16)} ${parseInt(hexMatch[3], 16)}`
              : '157 78 221';

            // Template label
            let labelText = goal.label || goal.variable;
            const labelMatches = labelText.match(/\{\{([^}]+)\}\}/g);
            if (labelMatches) {
              for (const match of labelMatches) {
                const varName = match.slice(2, -2).trim();
                if (varName === 'step') {
                  labelText = labelText.replace(match, String(isMultistep && steps ? Math.min(stepIndex + 1, steps.length) : 1));
                } else if (varName === 'total_steps') {
                  labelText = labelText.replace(match, String(isMultistep && steps ? steps.length : 1));
                } else if (varName === 'target') {
                  labelText = labelText.replace(match, goal.target.toLocaleString());
                } else if (varName === 'step_target') {
                  labelText = labelText.replace(match, currentTarget.toLocaleString());
                } else {
                  const val = variables[varName] ?? 0;
                  labelText = labelText.replace(match, val.toLocaleString());
                }
              }
            }

            return (
              <div
                key={goal.id}
                style={
                  {
                    background: 'rgba(10,8,18,0.88)',
                    border: `1px solid rgba(157,78,221,${isMilestone ? '0.7' : '0.3'})`,
                    borderRadius: '10px',
                    padding: '12px 14px',
                    backdropFilter: 'blur(10px)',
                    animation: isMilestone
                      ? 'sg-milestone-flash-preview 1.6s ease-in-out infinite'
                      : 'none',
                    '--sg-accent-rgb-p': rgbStr,
                  } as React.CSSProperties
                }
              >
                {/* Header row */}
                <div
                  style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'baseline',
                    marginBottom: '8px',
                  }}
                >
                  <span
                    style={{
                      fontSize: '13px',
                      fontWeight: 600,
                      color: 'var(--text-main)',
                      overflow: 'hidden',
                      textOverflow: 'ellipsis',
                      whiteSpace: 'nowrap',
                    }}
                  >
                    {labelText}
                  </span>
                  {goal.show_numbers !== false && (
                    <span
                      style={{
                        fontSize: '11px',
                        color: 'var(--text-secondary)',
                        marginLeft: '8px',
                        flexShrink: 0,
                      }}
                    >
                      {current.toLocaleString()} / {currentTarget.toLocaleString()}
                    </span>
                  )}
                </div>

                {/* Progress track */}
                <div
                  style={{
                    width: '100%',
                    height: '10px',
                    background: 'rgba(255,255,255,0.08)',
                    borderRadius: '999px',
                    overflow: 'hidden',
                  }}
                >
                  <div
                    style={{
                      height: '100%',
                      width: `${pct}%`,
                      minWidth: pct > 0 ? '4px' : '0',
                      background: color,
                      borderRadius: '999px',
                      transition: 'width 0.6s cubic-bezier(0.25,0.46,0.45,0.94)',
                      animation: isMilestone
                        ? 'sg-fill-glow-preview 1.2s ease-in-out infinite'
                        : 'none',
                    }}
                  />
                </div>

                {/* Footer: percentage */}
                {goal.show_percentage !== false && (
                  <div
                    style={{
                      display: 'flex',
                      justifyContent: 'flex-end',
                      marginTop: '5px',
                    }}
                  >
                    <span
                      style={{
                        fontSize: '11px',
                        fontWeight: 700,
                        color: 'var(--text-secondary)',
                      }}
                    >
                      {pct}%
                    </span>
                  </div>
                )}
              </div>
            );
          })
        )}
      </div>
    </div>
  );
};
