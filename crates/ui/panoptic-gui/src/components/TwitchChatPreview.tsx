import React from 'react';

export interface ChatBadge {
  set_id: string;
  id: string;
  info: string;
}

export interface ChatMessageData {
  id: string;
  user_id: string;
  user_login: string;
  user_name: string;
  message: string;
  color: string;
  pronouns?: string;
  badges: ChatBadge[];
  is_mod: bool;
  is_sub: bool;
  is_vip: bool;
  is_broadcaster: bool;
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
  const showBadges = settings.show_badges ?? true;
  const template = settings.message_template || "{pronouns} {user}: {message}";

  const getMessageContent = (msg: ChatMessageData) => {
    let content = template
      .replace('{user}', `<span class="chat-username" style="color: ${msg.color}">${msg.user_name}</span>`)
      .replace('{message}', `<span class="chat-text">${msg.message}</span>`)
      .replace('{pronouns}', (showPronouns && msg.pronouns) ? `<span class="chat-pronouns">[${msg.pronouns}]</span>` : '')
      .replace('{badges}', showBadges ? `<span class="chat-badges-wrap">${msg.badges.map(b => `<span class="badge-${b.set_id}">[${b.set_id[0].toUpperCase()}]</span>`).join('')}</span>` : '');
    
    return content;
  };

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
              className={`chat-message panoptic-overlay-card ${msg.is_mod ? 'chat-message-mod' : ''} ${msg.is_broadcaster ? 'chat-message-broadcaster' : ''} ${msg.is_vip ? 'chat-message-vip' : ''} ${msg.is_sub ? 'chat-message-sub' : ''}`}
              style={{ 
                width: '100%',
                padding: '10px',
                animation: 'chatSlideIn 0.3s ease both'
              }}
              dangerouslySetInnerHTML={{ __html: getMessageContent(msg) }}
            />
          ))
        )}
      </div>
    </div>
  );
};
