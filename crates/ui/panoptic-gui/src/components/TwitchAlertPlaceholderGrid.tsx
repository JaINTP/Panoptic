import React from 'react';

interface TwitchAlertPlaceholderGridProps {
  onInsertPlaceholder: (placeholder: string) => void;
}

export const TwitchAlertPlaceholderGrid: React.FC<TwitchAlertPlaceholderGridProps> = ({ onInsertPlaceholder }) => {
  const sections = [
    {
      title: 'Common Variables',
      items: [
        { code: '{user}', label: 'User Name' },
      ],
    },
    {
      title: 'Subscription Variables',
      items: [
        { code: '{tier}', label: 'Sub Tier (e.g. 1000)' },
        { code: '{months}', label: 'Total Months' },
        { code: '{total}', label: 'Gifts Total' },
      ],
    },
    {
      title: 'Raid Variables',
      items: [
        { code: '{viewers}', label: 'Viewer Count' },
      ],
    },
    {
      title: 'Cheer Variables',
      items: [
        { code: '{bits}', label: 'Bit Amount' },
        { code: '{message}', label: 'Cheer Message' },
      ],
    },
  ];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
      {sections.map((section) => (
        <div key={section.title}>
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
            {section.title}
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
            {section.items.map((item) => (
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
      ))}
    </div>
  );
};
