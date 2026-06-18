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
  is_mod: boolean;
  is_sub: boolean;
  is_vip: boolean;
  is_broadcaster: boolean;
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
  const animation = (settings.chat_animation || 'Slide').toLowerCase();
  const frameStyle = (settings.chat_frame_style || 'None').toLowerCase();
  const blur = settings.chat_background_blur || 0;

  const getMessageContent = (msg: ChatMessageData) => {
    const pronounsHtml = (showPronouns && msg.pronouns) ? `<span class="chat-pronouns">[${msg.pronouns}]</span>` : '';
    const badgesHtml = showBadges ? `<span class="chat-badges-wrap">${msg.badges.map(b => `<span class="badge-${b.set_id}">[${b.set_id[0].toUpperCase()}]</span>`).join('')}</span>` : '';

    let content = template
      .replace('{user}', `<span class="chat-username" style="color: ${msg.color}">${msg.user_name}</span>`)
      .replace('{message}', `<span class="chat-text">${msg.message}</span>`)
      .replace('{pronouns}', pronounsHtml)
      .replace('{badges}', badgesHtml);
    
    return content;
  };

  return (
    <div className="panoptic-overlay-wrapper twitch-chat-preview" style={{ 
        height: '300px', 
        overflowY: 'hidden', 
        display: 'flex', 
        flexDirection: 'column', 
        justifyContent: 'flex-end', 
        padding: '10px' 
    }}>
      <style>{`
        .preview-chat-message { backdrop-filter: blur(${blur}px); -webkit-backdrop-filter: blur(${blur}px); }
        .preview-anim-slide { animation: chatSlideIn 0.35s ease both; }
        .preview-anim-fade  { animation: chatFadeIn 0.4s ease both; }
        .preview-anim-pop   { animation: chatPopIn 0.3s ease both; }
        .preview-anim-bounce { animation: chatBounceIn 0.6s ease both; }
        
        .preview-frame-glass { background: rgba(255,255,255,0.05) !important; border: 1px solid rgba(255,255,255,0.1) !important; }
        .preview-frame-neon  { border: 2px solid var(--accent-primary) !important; box-shadow: 0 0 10px var(--accent-primary) !important; background: rgba(0,0,0,0.8) !important; }
        .preview-frame-retro { border: 3px double #fff !important; background: #000080 !important; border-radius: 0 !important; }
      `}</style>
      <div className="chat-stack" style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
        {messages.length === 0 ? (
          <div style={{ textAlign: 'center', opacity: 0.5, fontSize: '12px', fontStyle: 'italic', color: 'var(--text-secondary)' }}>
            No recent messages...
          </div>
        ) : (
          messages.slice(-4).map((msg) => (
            <div 
              key={msg.id} 
              className={`chat-message preview-chat-message panoptic-overlay-card preview-anim-${animation} ${frameStyle !== 'none' ? `preview-frame-${frameStyle}` : ''}`}
              style={{ 
                width: '100%',
                padding: '10px'
              }}
              dangerouslySetInnerHTML={{ __html: getMessageContent(msg) }}
            />
          ))
        )}
      </div>
    </div>
  );
};
