/**
 * StreamGoalsConfig
 *
 * Full configuration panel for the Stream Goals overlay:
 * - Built-in goal quick-adds (follower, sub, bits, raid, hype train)
 * - Goal list editor (add / edit / delete / reorder)
 * - Custom variable definitions with manual increment / decrement / reset
 * - Session stats reset button
 */
import React, { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { GoalConfig, CustomVar, SessionStats } from './StreamGoalsPreview';

// ─── Variable catalogue ───────────────────────────────────────────────────────

const BUILTIN_VARS = [
  { name: 'followers',       label: 'Followers this session',    icon: '👥' },
  { name: 'subscribers',     label: 'Subscribers this session',  icon: '⭐' },
  { name: 'bits',            label: 'Bits cheered this session', icon: '💎' },
  { name: 'raids',           label: 'Raids this session',        icon: '⚔️' },
  { name: 'gift_subs',       label: 'Gift subs this session',    icon: '🎁' },
  { name: 'hype_train_level',label: 'Hype Train level',          icon: '🚂' },
  { name: 'cheers_count',    label: 'Cheer events (count)',      icon: '🎊' },
  { name: 'redemptions',     label: 'Channel-point redemptions', icon: '🏆' },
  { name: 'chat_messages',   label: 'Chat messages this session',icon: '💬' },
  { name: 'unique_chatters', label: 'Unique chatters',           icon: '🧑‍🤝‍🧑' },
  { name: 'new_chatters',    label: 'First-time chatters',       icon: '👋' },
  { name: 'viewer_count',    label: 'Current viewer count',      icon: '👁️' },
] as const;

const PRESET_COLORS = [
  '#9147ff', '#06b6d4', '#10b981', '#f59e0b',
  '#ef4444', '#ec4899', '#8b5cf6', '#f97316',
];

// ─── Helpers ──────────────────────────────────────────────────────────────────

function uid(): string {
  return `goal-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`;
}

function makeDefaultGoal(variable: string, label: string, color: string): GoalConfig {
  return {
    id: uid(),
    label,
    variable,
    target: 50,
    color,
    show_percentage: true,
    show_numbers: true,
    milestone_celebration: true,
    enabled: true,
  };
}

// ─── Props ────────────────────────────────────────────────────────────────────

interface StreamGoalsConfigProps {
  goals: GoalConfig[];
  customVars: CustomVar[];
  stats: SessionStats;
  onGoalsChange: (goals: GoalConfig[]) => void;
  onCustomVarsChange: (vars: CustomVar[]) => void;
  onStatsReset: () => void;
}

// ─── Main component ───────────────────────────────────────────────────────────

export const StreamGoalsConfig: React.FC<StreamGoalsConfigProps> = ({
  goals,
  customVars,
  stats,
  onGoalsChange,
  onCustomVarsChange,
  onStatsReset,
}) => {
  const [editingGoalId, setEditingGoalId] = useState<string | null>(null);
  const [newVarName, setNewVarName] = useState('');
  const [newVarStep, setNewVarStep] = useState('1');

  // ── Goal persistence ────────────────────────────────────────────────────────

  const persistGoals = useCallback(async (updated: GoalConfig[]) => {
    onGoalsChange(updated);
    try {
      await invoke('save_goals_config', { goals: updated });
    } catch (e) {
      console.error('Failed to save goals:', e);
    }
  }, [onGoalsChange]);

  const persistCustomVars = useCallback(async (updated: CustomVar[]) => {
    onCustomVarsChange(updated);
    try {
      await invoke('save_custom_vars', { customVars: updated });
    } catch (e) {
      console.error('Failed to save custom vars:', e);
    }
  }, [onCustomVarsChange]);

  // ── Goal actions ─────────────────────────────────────────────────────────────

  const addQuickGoal = (variable: string, label: string) => {
    const idx = goals.length % PRESET_COLORS.length;
    persistGoals([...goals, makeDefaultGoal(variable, label, PRESET_COLORS[idx])]);
  };

  const updateGoal = (id: string, patch: Partial<GoalConfig>) => {
    persistGoals(goals.map((g) => (g.id === id ? { ...g, ...patch } : g)));
  };

  const deleteGoal = (id: string) => {
    persistGoals(goals.filter((g) => g.id !== id));
    if (editingGoalId === id) setEditingGoalId(null);
  };

  const moveGoal = (id: string, dir: -1 | 1) => {
    const idx = goals.findIndex((g) => g.id === id);
    if (idx < 0) return;
    const next = idx + dir;
    if (next < 0 || next >= goals.length) return;
    const arr = [...goals];
    [arr[idx], arr[next]] = [arr[next], arr[idx]];
    persistGoals(arr);
  };

  // ── Custom var actions ───────────────────────────────────────────────────────

  const addCustomVar = () => {
    const name = newVarName.trim().replace(/\s+/g, '_').toLowerCase();
    if (!name) return;
    if (customVars.some((v) => v.name === name)) return;
    const step = parseFloat(newVarStep) || 1;
    persistCustomVars([...customVars, { name, value: 0, step }]);
    setNewVarName('');
    setNewVarStep('1');
  };

  const removeCustomVar = (name: string) => {
    persistCustomVars(customVars.filter((v) => v.name !== name));
  };

  const changeCustomVarValue = async (name: string, op: 'increment' | 'decrement' | 'reset') => {
    try {
      const newVal = await invoke<number>('update_custom_var', { name, op });
      onCustomVarsChange(customVars.map((v) => (v.name === name ? { ...v, value: newVal } : v)));
    } catch (e) {
      console.error(`Failed to ${op} var ${name}:`, e);
    }
  };

  // ── Session reset ─────────────────────────────────────────────────────────────

  const handleResetSession = async () => {
    try {
      await invoke('reset_stream_goals_session');
      onStatsReset();
    } catch (e) {
      console.error('Failed to reset session:', e);
    }
  };

  // ── Styles ────────────────────────────────────────────────────────────────────

  const cardStyle: React.CSSProperties = {
    background: 'rgba(0,0,0,0.25)',
    border: '1px solid var(--border)',
    borderRadius: '8px',
    padding: '12px',
    marginBottom: '10px',
  };


  const inputStyle: React.CSSProperties = {
    width: '100%',
    background: 'rgba(0,0,0,0.4)',
    border: '1px solid var(--border)',
    borderRadius: '5px',
    padding: '6px 8px',
    color: 'var(--text-main)',
    fontSize: '12px',
    outline: 'none',
  };

  const btnStyle = (variant: 'primary' | 'ghost' | 'danger' | 'small'): React.CSSProperties => ({
    padding: variant === 'small' ? '3px 8px' : '6px 12px',
    fontSize: variant === 'small' ? '11px' : '12px',
    borderRadius: '5px',
    border: variant === 'danger'
      ? '1px solid rgba(244,63,94,0.4)'
      : '1px solid var(--border)',
    background: variant === 'primary'
      ? 'rgba(157,78,221,0.25)'
      : variant === 'danger'
      ? 'rgba(244,63,94,0.12)'
      : 'rgba(255,255,255,0.05)',
    color: variant === 'danger' ? '#f43f5e' : 'var(--text-main)',
    cursor: 'pointer',
    transition: 'opacity 0.15s',
  });

  // ── Render ────────────────────────────────────────────────────────────────────

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>

      {/* ── Live Session Stats ── */}
      <div>
        <h2 className="section-title">Live Session Stats</h2>
        <div style={cardStyle}>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '8px', marginBottom: '12px' }}>
            {BUILTIN_VARS.map((v) => {
              const val = (stats as any)[v.name] ?? 0;
              return (
                <div key={v.name} style={{
                  background: 'rgba(157,78,221,0.06)',
                  border: '1px solid rgba(157,78,221,0.15)',
                  borderRadius: '6px',
                  padding: '6px 8px',
                  fontSize: '11px',
                }}>
                  <div style={{ color: 'var(--text-secondary)', marginBottom: '2px' }}>
                    {v.icon} {v.label}
                  </div>
                  <div style={{ fontWeight: 700, color: 'var(--accent-primary)', fontSize: '14px' }}>
                    {typeof val === 'number' ? val.toLocaleString() : val || '—'}
                  </div>
                </div>
              );
            })}
          </div>
          <button style={btnStyle('danger')} onClick={handleResetSession}>
            ↺ Reset Session Counters
          </button>
        </div>
      </div>

      {/* ── Quick-add Built-in Goals ── */}
      <div>
        <h2 className="section-title">Quick-Add Built-in Goals</h2>
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: '6px' }}>
          {BUILTIN_VARS.map((v) => (
            <button
              key={v.name}
              style={{
                ...btnStyle('ghost'),
                fontSize: '11px',
                padding: '4px 10px',
              }}
              onClick={() => addQuickGoal(v.name, `${v.icon} ${v.label}`)}
            >
              + {v.icon} {v.label}
            </button>
          ))}
        </div>
      </div>

      {/* ── Goal List ── */}
      <div>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '10px' }}>
          <h2 className="section-title" style={{ margin: 0 }}>Goals ({goals.length})</h2>
          <button
            style={btnStyle('primary')}
            onClick={() => {
              const g = makeDefaultGoal('followers', '👥 Follower Goal', '#9147ff');
              persistGoals([...goals, g]);
              setEditingGoalId(g.id);
            }}
          >
            + Add Custom Goal
          </button>
        </div>

        {goals.length === 0 && (
          <div style={{ color: 'var(--text-secondary)', fontSize: '13px', fontStyle: 'italic' }}>
            No goals yet — use the quick-add buttons above or add a custom goal.
          </div>
        )}

        {goals.map((goal, idx) => (
          <GoalEditor
            key={goal.id}
            goal={goal}
            isFirst={idx === 0}
            isLast={idx === goals.length - 1}
            isOpen={editingGoalId === goal.id}
            onToggle={() => setEditingGoalId(editingGoalId === goal.id ? null : goal.id)}
            onChange={(patch) => updateGoal(goal.id, patch)}
            onDelete={() => deleteGoal(goal.id)}
            onMoveUp={() => moveGoal(goal.id, -1)}
            onMoveDown={() => moveGoal(goal.id, 1)}
            customVarNames={customVars.map((v) => v.name)}
          />
        ))}
      </div>

      {/* ── Custom Variables ── */}
      <div>
        <h2 className="section-title">Custom Variables</h2>
        <div style={cardStyle}>
          <p style={{ fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '10px' }}>
            Create your own counters (e.g. "deaths", "wins") and manually adjust them
            using the buttons. Use the variable name in any goal's "Variable" field.
          </p>

          {/* Add new */}
          <div style={{ display: 'flex', gap: '6px', marginBottom: '12px' }}>
            <div style={{ flex: 2 }}>
              <input
                style={inputStyle}
                placeholder="Variable name (e.g. deaths)"
                value={newVarName}
                onChange={(e) => setNewVarName(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && addCustomVar()}
              />
            </div>
            <div style={{ flex: 1 }}>
              <input
                style={{ ...inputStyle, textAlign: 'center' }}
                placeholder="Step"
                type="number"
                min="0.01"
                step="1"
                value={newVarStep}
                onChange={(e) => setNewVarStep(e.target.value)}
              />
            </div>
            <button style={btnStyle('primary')} onClick={addCustomVar}>
              Add
            </button>
          </div>

          {customVars.length === 0 && (
            <div style={{ color: 'var(--text-secondary)', fontSize: '12px', fontStyle: 'italic' }}>
              No custom variables yet.
            </div>
          )}

          {customVars.map((cv) => (
            <div key={cv.name} style={{
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
              padding: '8px 10px',
              background: 'rgba(157,78,221,0.06)',
              border: '1px solid rgba(157,78,221,0.15)',
              borderRadius: '6px',
              marginBottom: '6px',
            }}>
              <code style={{ flex: 1, fontSize: '12px', color: 'var(--accent-primary)' }}>
                {'{{'}{cv.name}{'}}'}
              </code>
              <span style={{ fontSize: '14px', fontWeight: 700, minWidth: '40px', textAlign: 'right', color: 'var(--text-main)' }}>
                {cv.value}
              </span>
              <button style={btnStyle('small')} onClick={() => changeCustomVarValue(cv.name, 'decrement')}>−</button>
              <button style={btnStyle('small')} onClick={() => changeCustomVarValue(cv.name, 'increment')}>+</button>
              <button style={btnStyle('small')} onClick={() => changeCustomVarValue(cv.name, 'reset')}>↺</button>
              <button style={{ ...btnStyle('small'), color: '#f43f5e', borderColor: 'rgba(244,63,94,0.3)' }}
                onClick={() => removeCustomVar(cv.name)}>✕</button>
            </div>
          ))}
        </div>
      </div>

      {/* ── Variable Reference ── */}
      <div>
        <h2 className="section-title">Variable Reference</h2>
        <div style={cardStyle}>
          <p style={{ fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '10px' }}>
            Use these names in the "Variable" field of any goal:
          </p>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: '6px' }}>
            {BUILTIN_VARS.map((v) => (
              <code key={v.name} style={{
                fontSize: '11px',
                background: 'rgba(157,78,221,0.1)',
                border: '1px solid rgba(157,78,221,0.2)',
                borderRadius: '4px',
                padding: '2px 7px',
                color: 'var(--accent-primary)',
              }}>
                {v.name}
              </code>
            ))}
            {customVars.map((v) => (
              <code key={v.name} style={{
                fontSize: '11px',
                background: 'rgba(6,182,212,0.1)',
                border: '1px solid rgba(6,182,212,0.2)',
                borderRadius: '4px',
                padding: '2px 7px',
                color: 'var(--accent-secondary)',
              }}>
                {v.name}
              </code>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};

// ─── GoalEditor sub-component ─────────────────────────────────────────────────

interface GoalEditorProps {
  goal: GoalConfig;
  isFirst: boolean;
  isLast: boolean;
  isOpen: boolean;
  onToggle: () => void;
  onChange: (patch: Partial<GoalConfig>) => void;
  onDelete: () => void;
  onMoveUp: () => void;
  onMoveDown: () => void;
  customVarNames: string[];
}

const GoalEditor: React.FC<GoalEditorProps> = ({
  goal, isFirst, isLast, isOpen, onToggle, onChange, onDelete, onMoveUp, onMoveDown, customVarNames,
}) => {
  const cardStyle: React.CSSProperties = {
    background: 'rgba(0,0,0,0.3)',
    border: `1px solid ${isOpen ? 'rgba(157,78,221,0.5)' : 'var(--border)'}`,
    borderRadius: '8px',
    marginBottom: '8px',
    overflow: 'hidden',
    transition: 'border-color 0.2s',
  };

  const inputStyle: React.CSSProperties = {
    width: '100%',
    background: 'rgba(0,0,0,0.4)',
    border: '1px solid var(--border)',
    borderRadius: '5px',
    padding: '6px 8px',
    color: 'var(--text-main)',
    fontSize: '12px',
    outline: 'none',
  };

  const fieldLabel = (text: string) => (
    <label style={{
      fontSize: '10px',
      fontWeight: 600,
      color: 'var(--text-secondary)',
      textTransform: 'uppercase',
      letterSpacing: '0.06em',
      display: 'block',
      marginBottom: '3px',
    }}>
      {text}
    </label>
  );

  const BUILTIN_VAR_NAMES = [
    'followers', 'subscribers', 'bits', 'raids', 'hosts', 'gift_subs',
    'chat_messages', 'unique_chatters', 'new_chatters', 'hype_train_level',
    'cheers_count', 'redemptions', 'viewer_count',
  ];

  return (
    <div style={cardStyle}>
      {/* Header */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          padding: '10px 12px',
          cursor: 'pointer',
          userSelect: 'none',
        }}
        onClick={onToggle}
      >
        <div
          style={{
            width: '10px',
            height: '10px',
            borderRadius: '50%',
            background: goal.color || '#9d4edd',
            flexShrink: 0,
          }}
        />
        <div style={{ flex: 1, fontSize: '13px', fontWeight: 600, color: goal.enabled ? 'var(--text-main)' : 'var(--text-muted)' }}>
          {goal.label || goal.variable}
        </div>
        <span style={{ fontSize: '11px', color: 'var(--text-secondary)' }}>
          {goal.variable} → {goal.target}
        </span>
        <span style={{ fontSize: '12px', color: 'var(--text-secondary)', marginLeft: '4px' }}>
          {isOpen ? '▲' : '▼'}
        </span>
      </div>

      {/* Expanded editor */}
      {isOpen && (
        <div style={{ padding: '0 12px 12px', borderTop: '1px solid var(--border)' }}>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '10px', marginTop: '10px' }}>

            {/* Label */}
            <div style={{ gridColumn: '1/-1' }}>
              {fieldLabel('Label')}
              <input
                style={inputStyle}
                value={goal.label}
                onChange={(e) => onChange({ label: e.target.value })}
                placeholder="e.g. 👥 Follower Goal"
              />
            </div>

            {/* Variable */}
            <div>
              {fieldLabel('Variable')}
              <select
                style={{ ...inputStyle, appearance: 'none' }}
                value={goal.variable}
                onChange={(e) => onChange({ variable: e.target.value })}
              >
                <optgroup label="Session Counters">
                  {BUILTIN_VAR_NAMES.map((n) => (
                    <option key={n} value={n}>{n}</option>
                  ))}
                </optgroup>
                {customVarNames.length > 0 && (
                  <optgroup label="Custom Variables">
                    {customVarNames.map((n) => (
                      <option key={n} value={n}>{n}</option>
                    ))}
                  </optgroup>
                )}
              </select>
            </div>

            {/* Target */}
            <div>
              {fieldLabel('Target')}
              <input
                style={inputStyle}
                type="number"
                min="1"
                step="1"
                value={goal.target}
                onChange={(e) => onChange({ target: parseFloat(e.target.value) || 1 })}
              />
            </div>

            {/* Color */}
            <div>
              {fieldLabel('Fill Colour')}
              <div style={{ display: 'flex', gap: '6px', alignItems: 'center' }}>
                <input
                  type="color"
                  value={goal.color}
                  onChange={(e) => onChange({ color: e.target.value })}
                  style={{ width: '36px', height: '30px', border: 'none', background: 'none', cursor: 'pointer', padding: 0 }}
                />
                <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
                  {PRESET_COLORS.map((c) => (
                    <div
                      key={c}
                      onClick={() => onChange({ color: c })}
                      title={c}
                      style={{
                        width: '16px',
                        height: '16px',
                        borderRadius: '50%',
                        background: c,
                        cursor: 'pointer',
                        border: goal.color === c ? '2px solid white' : '2px solid transparent',
                      }}
                    />
                  ))}
                </div>
              </div>
            </div>

            {/* Toggles */}
            <div>
              {fieldLabel('Options')}
              <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                {[
                  { key: 'show_percentage' as const,       label: 'Show percentage' },
                  { key: 'show_numbers' as const,           label: 'Show current / target' },
                  { key: 'milestone_celebration' as const, label: 'Milestone celebration' },
                  { key: 'enabled' as const,                label: 'Enabled in overlay' },
                ].map(({ key, label }) => (
                  <label key={key} style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '12px', cursor: 'pointer' }}>
                    <input
                      type="checkbox"
                      checked={goal[key] !== false}
                      onChange={(e) => onChange({ [key]: e.target.checked })}
                    />
                    {label}
                  </label>
                ))}
              </div>
            </div>
          </div>

          {/* Actions */}
          <div style={{ display: 'flex', gap: '6px', marginTop: '12px' }}>
            <button
              style={{ padding: '4px 8px', fontSize: '12px', borderRadius: '4px', border: '1px solid var(--border)', background: 'rgba(255,255,255,0.05)', cursor: 'pointer', color: 'var(--text-main)' }}
              disabled={isFirst}
              onClick={onMoveUp}
            >
              ↑
            </button>
            <button
              style={{ padding: '4px 8px', fontSize: '12px', borderRadius: '4px', border: '1px solid var(--border)', background: 'rgba(255,255,255,0.05)', cursor: 'pointer', color: 'var(--text-main)' }}
              disabled={isLast}
              onClick={onMoveDown}
            >
              ↓
            </button>
            <div style={{ flex: 1 }} />
            <button
              style={{ padding: '4px 10px', fontSize: '12px', borderRadius: '4px', border: '1px solid rgba(244,63,94,0.4)', background: 'rgba(244,63,94,0.1)', cursor: 'pointer', color: '#f43f5e' }}
              onClick={onDelete}
            >
              Delete Goal
            </button>
          </div>
        </div>
      )}
    </div>
  );
};
