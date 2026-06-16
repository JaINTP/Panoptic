import React from 'react';
import { invoke } from '@tauri-apps/api/core';
import { PlaybackState } from '../components/OverlayPreview';
import { PlaceholderGrid } from '../components/PlaceholderGrid';

interface OutputViewProps {
  template: string;
  setTemplate: (val: string) => void;
  playback: PlaybackState;
}

export const OutputView: React.FC<OutputViewProps> = ({
  template,
  setTemplate,
  playback,
}) => {
  const getFormatTime = (ms: number) => {
    if (isNaN(ms) || ms <= 0) return '0:00';
    const totalSecs = Math.floor(ms / 1000);
    const hours = Math.floor(totalSecs / 3600);
    const mins = Math.floor((totalSecs % 3600) / 60);
    const secs = totalSecs % 60;
    const pad = (num: number) => num.toString().padStart(2, '0');
    if (hours > 0) {
      return `${hours}:${pad(mins)}:${pad(secs)}`;
    }
    return `${mins}:${pad(secs)}`;
  };

  const getComponents = (ms: number) => {
    const totalSecs = Math.floor((ms || 0) / 1000);
    const h = Math.floor(totalSecs / 3600);
    const m = Math.floor((totalSecs % 3600) / 60);
    const s = totalSecs % 60;
    const mTotal = Math.floor(totalSecs / 60);
    const pad = (num: number) => num.toString().padStart(2, '0');
    return {
      h: h.toString(),
      m: pad(m),
      s: pad(s),
      mRaw: m.toString(),
      sRaw: s.toString(),
      mTotal: mTotal.toString(),
      mTotalPadded: pad(mTotal),
      sTotal: totalSecs.toString(),
    };
  };

  const pComp = getComponents(playback.progress_ms);
  const dComp = getComponents(playback.duration_ms);
  const pFormatted = getFormatTime(playback.progress_ms);
  const dFormatted = getFormatTime(playback.duration_ms);

  const formattedOutput = template
    .replace(/{title}/g, playback.title || '')
    .replace(/{artist}/g, playback.artist || '')
    .replace(/{album}/g, playback.album || '')
    .replace(/{progress_ms}/g, String(playback.progress_ms || 0))
    .replace(/{duration_ms}/g, String(playback.duration_ms || 0))
    .replace(/{progress}/g, pFormatted)
    .replace(/{duration}/g, dFormatted)
    .replace(/{progress_h}/g, pComp.h)
    .replace(/{progress_m}/g, pComp.m)
    .replace(/{progress_s}/g, pComp.s)
    .replace(/{progress_m_raw}/g, pComp.mRaw)
    .replace(/{progress_s_raw}/g, pComp.sRaw)
    .replace(/{progress_m_total}/g, pComp.mTotal)
    .replace(/{progress_m_total_padded}/g, pComp.mTotalPadded)
    .replace(/{progress_s_total}/g, pComp.sTotal)
    .replace(/{duration_h}/g, dComp.h)
    .replace(/{duration_m}/g, dComp.m)
    .replace(/{duration_s}/g, dComp.s)
    .replace(/{duration_m_raw}/g, dComp.mRaw)
    .replace(/{duration_s_raw}/g, dComp.sRaw)
    .replace(/{duration_m_total}/g, dComp.mTotal)
    .replace(/{duration_m_total_padded}/g, dComp.mTotalPadded)
    .replace(/{duration_s_total}/g, dComp.sTotal);

  const insertPlaceholder = (placeholder: string) => {
    const textarea = document.querySelector('.code-editor') as HTMLTextAreaElement;
    if (textarea) {
      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      const text = textarea.value;
      const before = text.substring(0, start);
      const after = text.substring(end, text.length);
      const newVal = before + placeholder + after;
      setTemplate(newVal);
      invoke('set_output_template', { template: newVal }).catch((err) =>
        console.error('Failed to save template:', err)
      );
      setTimeout(() => {
        textarea.focus();
        textarea.selectionStart = textarea.selectionEnd = start + placeholder.length;
      }, 0);
    } else {
      const newVal = template + placeholder;
      setTemplate(newVal);
      invoke('set_output_template', { template: newVal }).catch((err) =>
        console.error('Failed to save template:', err)
      );
    }
  };

  return (
    <div className="view-pane view-pane-scrollable">
      <h1 className="view-title">Output Templating</h1>
      <div className="section" style={{ marginBottom: '16px' }}>
        <h2 className="section-title">Template String</h2>
        <textarea
          className="code-editor"
          style={{ height: '80px', marginBottom: '8px' }}
          value={template}
          onChange={(e) => {
            const val = e.target.value;
            setTemplate(val);
            invoke('set_output_template', { template: val }).catch((err) =>
              console.error('Failed to save template:', err)
            );
          }}
        />
      </div>

      <div className="section" style={{ marginBottom: '16px' }}>
        <h2 className="section-title">Available Placeholders (Click to Insert)</h2>
        <PlaceholderGrid onInsertPlaceholder={insertPlaceholder} />
      </div>

      <div className="section" style={{ marginBottom: '16px' }}>
        <h2 className="section-title">Resulting Message Preview</h2>
        <div
          style={{
            padding: '16px',
            borderRadius: '8px',
            border: '1px solid var(--border)',
            backgroundColor: 'var(--bg-card)',
            fontFamily: 'monospace',
            fontSize: '14px',
            color: 'var(--text-main)',
            minHeight: '50px',
            display: 'flex',
            alignItems: 'center',
          }}
        >
          {formattedOutput || (
            <span style={{ color: 'var(--text-muted)' }}>(Empty template output)</span>
          )}
        </div>
      </div>
    </div>
  );
};
