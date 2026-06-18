import React from 'react';

interface TwitchChatPlaceholderGridProps {
  onInsertPlaceholder: (placeholder: string) => void;
}

export const TwitchChatPlaceholderGrid: React.FC<TwitchChatPlaceholderGridProps> = ({ onInsertPlaceholder }) => {
  const items = [
    { code: '{user}', label: 'Chatter Name' },
    { code: '{message}', label: 'Message Text' },
    { code: '{pronouns}', label: 'User Pronouns' },
    { code: '{badges}', label: 'Twitch Badges' },
    { code: '{color}', label: 'User Color' },
  ];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
      <h3
        style={{
          fontSize: '10px',
          color: 'var(--text-muted)',
          fontWeight: '700',
          marginBottom: '4px',
          textTransform: 'uppercase',
          letterSpacing: '0.05em',
        }}
      >
        Chat Variables
      </h3>
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))',
          gap: '8px',
          padding: '8px',
          borderRadius: '6px',
          border: '1px solid var(--border)',
          backgroundColor: 'rgba(0, 0, 0, 0.2)',
        }}
      >
        {items.map((item) => (
          <div
            key={item.code}
            className="placeholder-item"
            onClick={() => onInsertPlaceholder(item.code)}
            style={{ 
                display: 'flex', 
                flexDirection: 'column', 
                cursor: 'pointer',
                padding: '4px 8px',
                borderRadius: '4px'
            }}
            title="Click to insert"
          >
            <code
              style={{
                color: 'var(--accent-primary)',
                fontFamily: 'monospace',
                fontSize: '11px',
                fontWeight: '700',
              }}
            >
              {item.code}
            </code>
            <span style={{ fontSize: '10px', color: 'var(--text-secondary)' }}>{item.label}</span>
          </div>
        ))}
      </div>
    </div>
  );
};
