import React from 'react';

export interface ChatMessageData {
  id: string;
  user_id: string;
  user_login: string;
  user_name: string;
  message: string;
  color: string;
  pronouns?: string;
  timestamp: number;
}

export interface ChatState {
  messages: ChatMessageData[];
}

interface TwitchChatPreviewProps {
  state: ChatState;
  settings: Record<string, any>;
}

export const TwitchChatPreview: React.FC<TwitchChatPreviewProps> = ({ state, settings }) => {
  const messages = state.messages || [];
  const showPronouns = settings.show_pronouns ?? true;

  return (
    <div className="panoptic-overlay-wrapper twitch-chat-preview" style={{ height: '300px', overflowY: 'hidden', display: 'flex', flexDirection: 'column', justifyContent: 'flex-end', padding: '10px' }}>
      <div className="chat-stack" style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
        {messages.length === 0 ? (
          <div style={{ textAlign: 'center', opacity: 0.5, fontSize: '12px', fontStyle: 'italic', color: 'var(--text-secondary)' }}>
            No recent messages...
          </div>
        ) : (
          messages.slice(-4).map((msg) => (
            <div 
              key={msg.id} 
              className="chat-message panoptic-overlay-card"
              style={{ 
                width: '100%',
                padding: '10px',
                animation: 'chatSlideIn 0.3s ease both'
              }}
            >
              <div className="chat-header" style={{ marginBottom: '4px', display: 'flex', alignItems: 'baseline', gap: '6px' }}>
                {showPronouns && msg.pronouns && (
                  <span className="chat-pronouns" style={{ fontSize: '10px', opacity: 0.6, fontWeight: 700 }}>
                    [{msg.pronouns}]
                  </span>
                )}
                <span className="chat-username" style={{ color: msg.color, fontWeight: 900, fontSize: '13px' }}>
                  {msg.user_name}
                </span>
              </div>
              <div className="chat-text" style={{ fontSize: '12px', line_height: '1.4' }}>
                {msg.message}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};
