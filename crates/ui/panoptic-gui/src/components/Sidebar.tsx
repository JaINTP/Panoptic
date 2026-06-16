import React from 'react';
import { Monitor, HardDrive, ShieldCheck, Type } from 'lucide-react';

export type View = 'storage' | 'auth' | 'output' | 'display';

interface SidebarProps {
  activeView: View;
  setActiveView: (view: View) => void;
  updateVersion: string | null;
  handleOpenUpdate: () => void;
}

export const Sidebar: React.FC<SidebarProps> = ({
  activeView,
  setActiveView,
  updateVersion,
  handleOpenUpdate,
}) => {
  return (
    <nav className="sidebar">
      <div className="sidebar-title">PANOPTIC v0.1.3</div>
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
      {updateVersion && (
        <div
          onClick={handleOpenUpdate}
          style={{
            margin: 'auto 14px 14px 14px',
            padding: '10px 12px',
            borderRadius: '8px',
            background: 'linear-gradient(135deg, rgba(139, 92, 246, 0.15) 0%, rgba(167, 139, 250, 0.08) 100%)',
            border: '1px solid rgba(139, 92, 246, 0.35)',
            cursor: 'pointer',
            display: 'flex',
            flexDirection: 'column',
            gap: '4px',
            boxShadow: '0 4px 12px rgba(0, 0, 0, 0.25)',
            transition: 'all 0.2s ease',
            textAlign: 'left',
          }}
        >
          <span style={{ fontSize: '10px', fontWeight: '700', color: 'var(--accent-primary)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
            Update Available
          </span>
          <span style={{ fontSize: '12.5px', fontWeight: '600', color: '#ffffff' }}>
            Version {updateVersion}
          </span>
          <span style={{ fontSize: '10.5px', color: 'var(--text-secondary)' }}>
            Click to view release
          </span>
        </div>
      )}
    </nav>
  );
};
