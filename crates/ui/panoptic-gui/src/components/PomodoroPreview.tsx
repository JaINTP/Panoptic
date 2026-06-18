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

const PHASE_LABELS: Record<PomodoroState['phase'], string> = {
  work: 'Work',
  short_break: 'Short Break',
  long_break: 'Long Break',
};

const PHASE_COLORS: Record<PomodoroState['phase'], { ring: string; label: string; dot: string }> = {
  work:        { ring: '#7c3aed', label: '#a78bfa', dot: '#7c3aed' },
  short_break: { ring: '#059669', label: '#34d399', dot: '#059669' },
  long_break:  { ring: '#2563eb', label: '#60a5fa', dot: '#2563eb' },
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
  const colors   = PHASE_COLORS[state.phase] ?? PHASE_COLORS.work;
  const pct      = state.total_secs > 0 ? state.remaining_secs / state.total_secs : 0;
  const R        = 52;
  const CIRC     = 2 * Math.PI * R;
  const offset   = CIRC * (1 - pct);
  const dotCount = state.sessions_before_long_break || 4;
  const filled   = state.completed_sessions % dotCount;

  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center',
      gap: '12px',
      padding: '20px 24px',
      background: 'rgba(15, 15, 20, 0.88)',
      border: `1px solid rgba(139, 92, 246, 0.35)`,
      borderRadius: '12px',
      backdropFilter: 'blur(12px)',
      boxShadow: '0 4px 32px rgba(0, 0, 0, 0.45)',
      minWidth: '180px',
    }}>
      {/* Phase label */}
      <span style={{
        fontFamily: 'Inter, sans-serif',
        fontSize: '10px',
        fontWeight: 700,
        letterSpacing: '0.12em',
        textTransform: 'uppercase',
        color: colors.label,
      }}>
        {PHASE_LABELS[state.phase]}
      </span>

      {/* Ring + time */}
      <div style={{ position: 'relative', width: 110, height: 110 }}>
        <svg
          viewBox="0 0 120 120"
          width={110}
          height={110}
          style={{ transform: 'rotate(-90deg)' }}
        >
          <circle
            cx={60} cy={60} r={R}
            fill="none"
            stroke="rgba(139, 92, 246, 0.15)"
            strokeWidth={6}
          />
          <circle
            cx={60} cy={60} r={R}
            fill="none"
            stroke={colors.ring}
            strokeWidth={6}
            strokeLinecap="round"
            strokeDasharray={CIRC}
            strokeDashoffset={offset}
            style={{ transition: 'stroke-dashoffset 0.9s linear' }}
          />
        </svg>

        {/* Centered time + status */}
        <div style={{
          position: 'absolute',
          inset: 0,
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          gap: '2px',
        }}>
          <span style={{
            fontFamily: '"JetBrains Mono", monospace',
            fontSize: '22px',
            fontWeight: 700,
            color: '#fff',
            lineHeight: 1,
            letterSpacing: '-0.02em',
          }}>
            {formatTime(state.remaining_secs)}
          </span>
          <span style={{
            fontFamily: 'Inter, sans-serif',
            fontSize: '8px',
            fontWeight: 600,
            letterSpacing: '0.1em',
            textTransform: 'uppercase',
            color: state.is_running ? '#4ade80' : 'rgba(255,255,255,0.4)',
          }}>
            {state.is_running ? 'Running' : 'Paused'}
          </span>
        </div>
      </div>

      {/* Session dots */}
      <div style={{ display: 'flex', gap: '6px', alignItems: 'center' }}>
        {Array.from({ length: dotCount }, (_, i) => (
          <div
            key={i}
            style={{
              width: 7,
              height: 7,
              borderRadius: '50%',
              background: i < filled ? colors.dot : 'rgba(139, 92, 246, 0.2)',
              border: `1px solid ${i < filled ? colors.dot : 'rgba(139, 92, 246, 0.35)'}`,
              transition: 'background 0.3s ease',
            }}
          />
        ))}
      </div>
    </div>
  );
};
