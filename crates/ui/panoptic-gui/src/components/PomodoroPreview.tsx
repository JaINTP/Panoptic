import React from 'react';

export interface PomodoroState {
  phase: 'work' | 'short_break' | 'long_break';
  remaining_secs: number;
  total_secs: number;
  is_running: boolean;
  completed_sessions: number;
  sessions_before_long_break: number;
  work_duration_mins: number;
  short_break_mins: number;
  long_break_mins: number;
}

export const DEFAULT_POMODORO_STATE: PomodoroState = {
  phase: 'work',
  remaining_secs: 25 * 60,
  total_secs: 25 * 60,
  is_running: false,
  completed_sessions: 0,
  sessions_before_long_break: 4,
  work_duration_mins: 25,
  short_break_mins: 5,
  long_break_mins: 15,
};

// Default CSS mirrors the inline <style> in pomodoro.html exactly.
// DisplayView injects this before the user's custom CSS so the cascade
// order matches the actual overlay: defaults first, custom rules win.
export const POMODORO_DEFAULT_CSS = `
  .pomodoro-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--pomodoro-card-gap, 12px);
    padding: var(--pomodoro-card-padding, 20px 24px);
    background: var(--pomodoro-card-bg, rgba(15, 15, 20, 0.88));
    border: var(--pomodoro-card-border-width, 1px) solid var(--pomodoro-card-border-color, rgba(139, 92, 246, 0.35));
    border-radius: var(--pomodoro-card-radius, 12px);
    backdrop-filter: blur(var(--pomodoro-card-blur, 12px));
    -webkit-backdrop-filter: blur(var(--pomodoro-card-blur, 12px));
    box-shadow: var(--pomodoro-card-shadow, 0 4px 32px rgba(0, 0, 0, 0.45));
    min-width: var(--pomodoro-card-min-width, 180px);
  }
  .pomodoro-phase {
    font-family: var(--pomodoro-phase-font, 'Inter', sans-serif);
    font-size: var(--pomodoro-phase-size, 11px);
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--pomodoro-phase-color, #a78bfa);
  }
  .pomodoro-ring-wrap {
    position: relative;
    width: var(--pomodoro-ring-size, 120px);
    height: var(--pomodoro-ring-size, 120px);
  }
  .pomodoro-ring-svg {
    width: 100%;
    height: 100%;
    transform: rotate(-90deg);
  }
  .pomodoro-ring-track {
    fill: none;
    stroke: var(--pomodoro-ring-track-color, rgba(139, 92, 246, 0.15));
    stroke-width: var(--pomodoro-ring-width, 6);
  }
  .pomodoro-ring-fill {
    fill: none;
    stroke: var(--pomodoro-ring-color, #7c3aed);
    stroke-width: var(--pomodoro-ring-width, 6);
    stroke-linecap: round;
    transition: stroke-dashoffset 0.9s linear;
  }
  .pomodoro-time-wrap {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
  }
  .pomodoro-time {
    font-family: var(--pomodoro-time-font, 'JetBrains Mono', monospace);
    font-size: var(--pomodoro-time-size, 26px);
    font-weight: 700;
    color: var(--pomodoro-time-color, #ffffff);
    line-height: 1;
    letter-spacing: -0.02em;
  }
  .pomodoro-status {
    font-family: var(--pomodoro-status-font, 'Inter', sans-serif);
    font-size: var(--pomodoro-status-size, 9px);
    font-weight: 600;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--pomodoro-status-color, rgba(255, 255, 255, 0.4));
  }
  .pomodoro-status.running {
    color: var(--pomodoro-status-running-color, #4ade80);
  }
  .pomodoro-dots {
    display: flex;
    gap: var(--pomodoro-dots-gap, 6px);
    align-items: center;
  }
  .pomodoro-dot {
    width: var(--pomodoro-dot-size, 8px);
    height: var(--pomodoro-dot-size, 8px);
    border-radius: 50%;
    background: var(--pomodoro-dot-empty-color, rgba(139, 92, 246, 0.2));
    border: 1px solid var(--pomodoro-dot-border-color, rgba(139, 92, 246, 0.35));
    transition: background 0.3s ease;
  }
  .pomodoro-dot.filled {
    background: var(--pomodoro-dot-filled-color, #7c3aed);
    border-color: var(--pomodoro-dot-filled-color, #7c3aed);
  }
  .pomodoro-card[data-phase="work"] {
    --pomodoro-phase-color: var(--pomodoro-work-phase-color, #a78bfa);
    --pomodoro-ring-color: var(--pomodoro-work-ring-color, #7c3aed);
    --pomodoro-dot-filled-color: var(--pomodoro-work-dot-color, #7c3aed);
  }
  .pomodoro-card[data-phase="short_break"] {
    --pomodoro-phase-color: var(--pomodoro-short-break-phase-color, #34d399);
    --pomodoro-ring-color: var(--pomodoro-short-break-ring-color, #059669);
    --pomodoro-dot-filled-color: var(--pomodoro-short-break-dot-color, #059669);
  }
  .pomodoro-card[data-phase="long_break"] {
    --pomodoro-phase-color: var(--pomodoro-long-break-phase-color, #60a5fa);
    --pomodoro-ring-color: var(--pomodoro-long-break-ring-color, #2563eb);
    --pomodoro-dot-filled-color: var(--pomodoro-long-break-dot-color, #2563eb);
  }
`;

const PHASE_LABELS: Record<PomodoroState['phase'], string> = {
  work: 'Work',
  short_break: 'Short Break',
  long_break: 'Long Break',
};

function formatTime(secs: number): string {
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${m}:${s.toString().padStart(2, '0')}`;
}

interface PomodoroPreviewProps {
  state: PomodoroState;
}

export const PomodoroPreview: React.FC<PomodoroPreviewProps> = ({ state }) => {
  const R        = 52;
  const CIRC     = 2 * Math.PI * R;
  const pct      = state.total_secs > 0 ? state.remaining_secs / state.total_secs : 0;
  const offset   = CIRC * (1 - pct);
  const dotCount = state.sessions_before_long_break || 4;
  const filled   = state.completed_sessions % dotCount;

  return (
    <div className="pomodoro-card" data-phase={state.phase}>
      <span className="pomodoro-phase">
        {PHASE_LABELS[state.phase]}
      </span>

      <div className="pomodoro-ring-wrap">
        <svg className="pomodoro-ring-svg" viewBox="0 0 120 120">
          <circle className="pomodoro-ring-track" cx="60" cy="60" r={R} />
          <circle
            className="pomodoro-ring-fill"
            cx="60" cy="60" r={R}
            style={{ strokeDasharray: CIRC, strokeDashoffset: offset }}
          />
        </svg>
        <div className="pomodoro-time-wrap">
          <span className="pomodoro-time">{formatTime(state.remaining_secs)}</span>
          <span className={`pomodoro-status${state.is_running ? ' running' : ''}`}>
            {state.is_running ? 'Running' : 'Paused'}
          </span>
        </div>
      </div>

      <div className="pomodoro-dots">
        {Array.from({ length: dotCount }, (_, i) => (
          <div key={i} className={`pomodoro-dot${i < filled ? ' filled' : ''}`} />
        ))}
      </div>
    </div>
  );
};
