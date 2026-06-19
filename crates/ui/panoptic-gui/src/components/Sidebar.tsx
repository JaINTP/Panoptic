import React, { useState } from 'react';
import { Monitor, HardDrive, ShieldCheck, Type, X, ExternalLink } from 'lucide-react';

export type View = 'storage' | 'auth' | 'output' | 'display';

interface SidebarProps {
  activeView: View;
  setActiveView: (view: View) => void;
  updateVersion: string | null;
  handleOpenUpdate: () => void;
  version: string;
}

export const Sidebar: React.FC<SidebarProps> = ({
  activeView,
  setActiveView,
  updateVersion,
  handleOpenUpdate,
  version,
}) => {
  const [dismissed, setDismissed] = useState(false);
  const showUpdate = !!updateVersion && !dismissed;

  return (
    <nav className="sidebar">
      <div className="sidebar-title">PANOPTIC v{version}</div>
      <button
        type="button"
        className={`sidebar-item ${activeView === 'display' ? 'active' : ''}`}
        onClick={() => setActiveView('display')}
      >
        <Monitor size={18} /> Live Overlay
      </button>
      <button
        type="button"
        className={`sidebar-item ${activeView === 'storage' ? 'active' : ''}`}
        onClick={() => setActiveView('storage')}
      >
        <HardDrive size={18} /> Storage
      </button>
      <button
        type="button"
        className={`sidebar-item ${activeView === 'auth' ? 'active' : ''}`}
        onClick={() => setActiveView('auth')}
      >
        <ShieldCheck size={18} /> Auth
      </button>
      <button
        type="button"
        className={`sidebar-item ${activeView === 'output' ? 'active' : ''}`}
        onClick={() => setActiveView('output')}
      >
        <Type size={18} /> Output
      </button>

      {/* Update link - shown only when a newer GitHub release exists and not dismissed */}
      {showUpdate && (
        <div style={{
          marginTop: 'auto',
          marginBottom: '12px',
          marginLeft: '10px',
          marginRight: '10px',
          padding: '8px 10px',
          borderRadius: '6px',
          background: 'rgba(139, 92, 246, 0.08)',
          border: '1px solid rgba(139, 92, 246, 0.22)',
          display: 'flex',
          alignItems: 'center',
          gap: '6px',
        }}>
          <button
            type="button"
            onClick={handleOpenUpdate}
            title={`${updateVersion} is available on GitHub`}
            style={{
              flex: 1,
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              background: 'none',
              border: 'none',
              padding: 0,
              cursor: 'pointer',
              color: 'var(--accent-primary)',
              fontSize: '12px',
              fontWeight: 600,
              textAlign: 'left',
              minWidth: 0,
            }}
          >
            <ExternalLink size={12} style={{ flexShrink: 0, opacity: 0.8 }} />
            <span style={{
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
            }}>
              v{updateVersion} available
            </span>
          </button>
          <button
            type="button"
            aria-label="Dismiss update notification"
            onClick={(e) => { e.stopPropagation(); setDismissed(true); }}
            style={{
              flexShrink: 0,
              background: 'none',
              border: 'none',
              padding: '2px',
              cursor: 'pointer',
              color: 'var(--text-muted)',
              display: 'flex',
              alignItems: 'center',
              borderRadius: '3px',
              opacity: 0.6,
            }}
            onMouseEnter={(e) => { (e.currentTarget as HTMLButtonElement).style.opacity = '1'; }}
            onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.opacity = '0.6'; }}
          >
            <X size={12} />
          </button>
        </div>
      )}
    </nav>
  );
};
