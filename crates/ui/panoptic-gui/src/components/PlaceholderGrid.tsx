import React from 'react';

interface PlaceholderGridProps {
  onInsertPlaceholder: (placeholder: string) => void;
}

export const PlaceholderGrid: React.FC<PlaceholderGridProps> = ({ onInsertPlaceholder }) => {
  const sections = [
    {
      title: 'Metadata',
      items: [
        { code: '{title}', label: 'Track Title' },
        { code: '{artist}', label: 'Artist Name(s)' },
        { code: '{album}', label: 'Album Name' },
      ],
    },
    {
      title: 'Formatted Time (Recommended)',
      items: [
        { code: '{progress}', label: 'Smart Progress (e.g. 3:04)' },
        { code: '{duration}', label: 'Smart Duration (e.g. 4:12)' },
      ],
    },
    {
      title: 'Detailed Time (Progress)',
      items: [
        { code: '{progress_h}', label: 'Hours (unpadded)' },
        { code: '{progress_m}', label: 'Mins of hour (padded)' },
        { code: '{progress_s}', label: 'Secs of min (padded)' },
        { code: '{progress_m_raw}', label: 'Mins of hour (unpadded)' },
        { code: '{progress_s_raw}', label: 'Secs of min (unpadded)' },
        { code: '{progress_m_total}', label: 'Total minutes (unpadded)' },
        { code: '{progress_m_total_padded}', label: 'Total minutes (padded)' },
        { code: '{progress_s_total}', label: 'Total seconds (unpadded)' },
        { code: '{progress_ms}', label: 'Progress in milliseconds' },
      ],
    },
    {
      title: 'Detailed Time (Duration)',
      items: [
        { code: '{duration_h}', label: 'Hours (unpadded)' },
        { code: '{duration_m}', label: 'Mins of hour (padded)' },
        { code: '{duration_s}', label: 'Secs of min (padded)' },
        { code: '{duration_m_raw}', label: 'Mins of hour (unpadded)' },
        { code: '{duration_s_raw}', label: 'Secs of min (unpadded)' },
        { code: '{duration_m_total}', label: 'Total minutes (unpadded)' },
        { code: '{duration_m_total_padded}', label: 'Total minutes (padded)' },
        { code: '{duration_s_total}', label: 'Total seconds (unpadded)' },
        { code: '{duration_ms}', label: 'Duration in milliseconds' },
      ],
    },
  ];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
      {sections.map((section) => (
        <div key={section.title}>
          <h3
            style={{
              fontFamily: 'MedievalSharp, serif',
              fontSize: '12px',
              color: 'var(--text-secondary)',
              fontWeight: '700',
              marginBottom: '8px',
              textTransform: 'uppercase',
              letterSpacing: '0.08em',
            }}
          >
            {section.title}
          </h3>
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(160px, 1fr))',
              gap: '12px',
              padding: '12px 16px',
              borderRadius: '8px',
              border: '1px solid var(--border)',
              backgroundColor: 'rgba(5, 4, 8, 0.5)',
              boxShadow: 'inset 0 2px 6px rgba(0, 0, 0, 0.6)',
            }}
          >
            {section.items.map((item) => (
              <div
                key={item.code}
                className="placeholder-item"
                onClick={() => onInsertPlaceholder(item.code)}
                title="Click to insert at cursor"
              >
                <code
                  style={{
                    color: 'var(--accent-primary-hover)',
                    fontFamily: 'monospace',
                    fontSize: '12.5px',
                    fontWeight: '600',
                  }}
                >
                  {item.code}
                </code>
                <span style={{ fontSize: '11px', color: 'var(--text-secondary)' }}>{item.label}</span>
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
};
